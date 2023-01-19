use aiken_lang::ast::{ModuleKind, TypedFunction, UntypedModule};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::copy_aiken_project_lib::Project;
use aiken_project::config::Config;
use aiken_project::module::{CheckedModules, ParsedModule, ParsedModules};
use aiken_project::script::Script;
use aiken_project::telemetry::EventListener;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use ir_to_unode::UResult;
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::input::StringInput;
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, SourceMap};
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::ast::Program as SWCProgram;
use swc_core::ecma::parser::error::Error as SWCError;
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::parser::{PResult, Parser, Syntax, TsConfig};

use crate::ir::IR;
use crate::ir_to_unode::ModuleBuilderFromIR;
use crate::ir_visitor::IRVisitor;
use crate::js_to_ir::JsToIR;
use crate::program::{Error, Language, Source};
use crate::{ir_to_unode, js_compiler};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsError {
    pub error: String,
}

impl From<SWCError> for JsError {
    fn from(e: SWCError) -> Self {
        let error = e.kind().msg().to_string();
        JsError { error }
    }
}

impl From<SWCError> for Error {
    fn from(e: SWCError) -> Self {
        Error::Js(e.into())
    }
}

impl JsError {
    pub fn new(error: String) -> Self {
        JsError { error }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct JsSyntaxEsVersion {
    syntax: Syntax,
    es_version: EsVersion,
}

impl Default for JsSyntaxEsVersion {
    fn default() -> Self {
        JsSyntaxEsVersion {
            // Go full types with Typescript (vs Javascript).
            syntax: Syntax::Typescript(TsConfig::default()),
            // Use latest and greatest.
            es_version: EsVersion::Es2022,
        }
    }
}

/// Parsed program and comments
#[derive(Debug, Clone)]
pub struct ParsedJs {
    program: Rc<SWCProgram>,
    comments: Rc<SingleThreadedComments>,
}

impl ParsedJs {
    pub fn new(program_: SWCProgram, comments_: SingleThreadedComments) -> ParsedJs {
        let program = Rc::new(program_);
        let comments = Rc::new(comments_);
        ParsedJs { program, comments }
    }
}

pub fn parse_js(code: &str, fname: &Path, jssynesv: JsSyntaxEsVersion) -> PResult<ParsedJs> {
    let JsSyntaxEsVersion { syntax, es_version } = jssynesv;
    let filename = FileName::Real(fname.to_path_buf());
    let rc_source_map = Lrc::<SourceMap>::default();
    let rc_source_file = rc_source_map.new_source_file(filename, code.into());
    let input = StringInput::from(&*rc_source_file);
    let comments = SingleThreadedComments::default();

    // let compiler = swc::Compiler::new(rc_source_map);

    let lexer = Lexer::new(syntax, es_version, input, Some(&comments));

    let mut parser = Parser::new_from(lexer);

    let program = parser.parse_program()?;

    Ok(ParsedJs::new(program, comments))
}

fn parse_js_source(source: &JsSource) -> PResult<ParsedJs> {
    assert_eq!(source.lang, Language::Javascript);

    // let code = source.code.as_ref();
    // let jssynesv = JsSyntaxEsVersion::default();
    // parse_js_ex(code, &source.script_path, jssynesv)

    let JsSource {
        lang,
        extra,
        script_name,
        script_path,
        code,
    } = source;
    let JsSyntaxEsVersion { syntax, es_version } = extra; //.clone();

    let filename = FileName::Real(script_path.clone());
    let rc_source_map = Lrc::<SourceMap>::default();
    let rc_source_file = rc_source_map.new_source_file(filename, code.into());
    let input = StringInput::from(&*rc_source_file);
    let comments = SingleThreadedComments::default();

    // let compiler = swc::Compiler::new(rc_source_map);

    let lexer = Lexer::new(*syntax, *es_version, input, Some(&comments));

    let mut parser = Parser::new_from(lexer);

    let program = parser.parse_program()?;

    Ok(ParsedJs::new(program, comments))
}

pub type JsSource = Source<JsSyntaxEsVersion>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Javascript {
    source: JsSource,
}

impl From<aiken_project::error::Error> for Error {
    fn from(ae: aiken_project::error::Error) -> Self {
        Error::Aiken(ae)
    }
}

pub type Validators = Vec<(PathBuf, String, TypedFunction)>;

impl Javascript {
    fn new(source: JsSource) -> Self {
        Javascript { source }
    }

    fn name(&self) -> String {
        format!("{:?}", self)
    }

    fn parse_js_to_swc(&self) -> Result<ParsedJs, Error> {
        let parsed_js = parse_js_source(&self.source)?;
        Ok(parsed_js)
    }

    fn transform_swc_to_ir(&self, program: &SWCProgram) -> Result<IR, Error> {
        let ir = JsToIR::default().visit_program(program)?;
        Ok(ir)
    }

    fn parse_js_to_ir(&self) -> Result<IR, Error> {
        let ParsedJs { program, comments } = self.parse_js_to_swc()?;
        self.transform_swc_to_ir(program.as_ref())
    }

    fn transform_ir_to_aiken_untyped(&self, ir: &IR) -> Result<UntypedModule, Error> {
        let script_name = self
            .source
            .script_path
            .to_str()
            .ok_or_else(|| Error::BadFilename(self.source.script_path.to_path_buf()))?;
        let builder = <ModuleBuilderFromIR as IRVisitor<UResult>>::new();
        let unode = builder.visit_ir(ir)?;
        let module = unode.make_untyped_module(script_name.to_string())?;

        Ok(module)
    }

    /// Note: the basic logic is from Aiken's compilation pipeline, adjusted for our use-case.
    /// See [aiken_project::Project] and [aiken_project_copy::Project]
    fn transform_aiken_untyped_to_typed<T: EventListener>(
        &self,
        umod: &UntypedModule,
        event_listener: T,
    ) -> Result<(RefCell<Project<T>>, Validators, CheckedModules), Error> {
        let name = &self.source.script_name;
        let pmodule = ParsedModule {
            path: self.source.script_path.clone(),
            name: name.clone(),
            code: "".to_string(),
            kind: ModuleKind::Validator,
            package: "".to_string(),
            ast: umod.clone(),
            extra: Default::default(),
        };

        let umodmap = HashMap::from([(name.clone(), pmodule)]);
        let parsed_modules: ParsedModules = umodmap.into();

        let config = Config {
            name: name.clone(),
            version: "0.0.1".into(),
            description: name.clone(),
            repository: None,
        };
        let root = PathBuf::from("/");

        let mut project = Project::new(config, root, event_listener);

        let mut checked_modules = project.type_check(parsed_modules)?;
        let validators = project.validate_validators(&mut checked_modules)?;
        println!("Validators: {:?}", validators);

        let project = RefCell::new(project);

        Ok((project, validators, checked_modules))
    }

    pub fn gen_code<T: EventListener>(
        &self,
        project: &mut Project<T>,
        validators: Validators,
        checked_modules: &CheckedModules,
    ) -> Result<Vec<Script>, Error> {
        let programs = project.code_gen(validators, checked_modules)?;

        Ok(programs)
    }

    pub fn end_to_end<T: EventListener>(&self, event_listener: T) -> Result<Vec<Script>, Error> {
        // 1. Parse Javascript/Typescript source to `swc` AST.
        let ParsedJs { program, comments } = self.parse_js_to_swc()?;

        // 2. Transform `swc` AST to `jutus` IR.
        let ir = self.transform_swc_to_ir(program.as_ref())?;

        // 3. Transform `jutus` IR to `aiken` untyped trees (`UntypedModule`).
        let untyped_module = self.transform_ir_to_aiken_untyped(&ir)?;

        // 4. Transform `aiken` untyped to typed trees (`TypedModule`).
        let (project, validators, checked_modules) =
            self.transform_aiken_untyped_to_typed(&untyped_module, event_listener)?;

        // 5. Generate code
        let mut project = project.borrow_mut();
        self.gen_code(&mut project, validators, &checked_modules)
    }
}

/// Just a helper for writing main() functions in examples/
pub fn parser_main_helper(code: &str, fname: &Path) -> Result<(), Error> {
    let jssynesv = JsSyntaxEsVersion::default();
    let ParsedJs { program, comments } = js_compiler::parse_js(code, fname, jssynesv)?;

    println!();
    let pretty = serde_json::to_string_pretty(program.as_ref()).unwrap();
    println!("{}", pretty);
    println!();
    println!("comments = {:?}", comments);

    let path = fname.to_path_buf();
    let script_name = fname
        .to_str()
        .ok_or_else(|| Error::BadFilename(path.clone()))?
        .to_owned();
    let source = Source {
        lang: Language::Javascript,
        extra: jssynesv,
        script_name,
        script_path: path,
        code: code.to_string(),
    };

    println!();
    println!("============================");
    println!("=== IR =====================");
    let js = Javascript::new(source);
    let name = js.name();
    let ir = js.parse_js_to_ir()?;

    let ir_pretty = serde_json::to_string_pretty(&ir).unwrap();
    println!("{}", ir_pretty);

    println!();
    println!("============================");
    println!("=== Aiken Trees  ===========");

    println!();
    println!("============================");
    let umod = js.transform_ir_to_aiken_untyped(&ir)?;
    println!("UNTYPED {:?}", umod);

    println!();
    println!("============================");
    let listener = aiken::Terminal::default();
    let (project, validators, checked_modules) =
        js.transform_aiken_untyped_to_typed(&umod, listener)?;
    // Note Assuming an untyped module gives rise to a corresponding typed module
    let tmod = &checked_modules.values().next().unwrap().ast;
    // let tmod = tmod.clone();
    // println!("TYPED {:?}", tmod);
    for tdef in tmod.definitions.iter() {
        println!("TDEF {:?}", tdef);
    }

    println!();
    println!("============================");
    let mut project = project.borrow_mut();
    let scripts = js.gen_code(&mut project, validators, &checked_modules)?;
    for (index, script) in scripts.iter().enumerate() {
        println!("SCRIPT[{:?}] {:?}", index, script);
    }

    let eval_infos = project.eval_scripts(scripts, None);
    for (index, eval_info) in eval_infos.iter().enumerate() {
        println!("EvalInfo[{:?}] {:?}", index, eval_info);
    }

    Ok(())
}
