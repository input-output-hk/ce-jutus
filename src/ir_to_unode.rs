//! Transforms our IR to aiken's untyped trees: [UntypedDefinition] and [UntypedExpr].

use aiken_lang::ast::{
    Arg, ArgName, AssignmentKind, BinOp, Function, IfBranch, ModuleKind, Span, UntypedDefinition,
    UntypedModule, UntypedPattern,
};
use aiken_lang::expr::UntypedExpr;
use serde::{Deserialize, Serialize};
use vec1::vec1;

use crate::ir::*;
use crate::ir_visitor::IRVisitor;
use crate::program::Error;

/// Note this is clearly a hack
fn no_span() -> Span {
    Span { start: 0, end: 0 }
}

fn ir_bin_op_to_bin_op(op: IRBinOp) -> BinOp {
    // Note how the translation to Aiken assumes integer operations but JS has floats
    match op {
        IRBinOp::EqEq => BinOp::Eq,
        IRBinOp::NotEq => BinOp::NotEq,
        IRBinOp::Lt => BinOp::LtEqInt,
        IRBinOp::LtEq => BinOp::LtEqInt,
        IRBinOp::Gt => BinOp::GtInt,
        IRBinOp::GtEq => BinOp::GtEqInt,
        IRBinOp::Add => BinOp::AddInt,
        IRBinOp::Sub => BinOp::SubInt,
        IRBinOp::Mul => BinOp::MultInt,
        IRBinOp::Div => BinOp::DivInt,
        IRBinOp::Mod => BinOp::ModInt,
        IRBinOp::LogicalOr => BinOp::Or,
        IRBinOp::LogicalAnd => BinOp::And,
    }
}

#[derive(Debug)]
/// A small wrapper around the AST nodes from aiken that we are interested in
pub enum UNode {
    Expr(UntypedExpr),
    Def(UntypedDefinition),
    Script(Vec<UntypedDefinition>),
}

impl From<UntypedExpr> for UNode {
    fn from(ue: UntypedExpr) -> Self {
        UNode::Expr(ue)
    }
}

impl From<UntypedDefinition> for UNode {
    fn from(ud: UntypedDefinition) -> Self {
        UNode::Def(ud)
    }
}

impl From<Vec<UntypedDefinition>> for UNode {
    fn from(script: Vec<UntypedDefinition>) -> Self {
        UNode::Script(script)
    }
}

impl UNode {
    pub fn make_untyped_module(self, name: String) -> Result<UntypedModule, UError> {
        let definitions = self.to_script_result()?;
        let kind = ModuleKind::Validator;

        let module = UntypedModule {
            name,
            docs: vec![],
            type_info: (),
            definitions,
            kind,
        };

        Ok(module)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UError {
    ExpectingExpr,
    ExpectingDefinition,
    ExpectingScript,
    ExpectingFunDef,
}

impl From<UError> for Error {
    fn from(e: UError) -> Self {
        Error::U(e)
    }
}

#[derive(Default)]
pub struct ModuleBuilderFromIR;

pub type UResult = Result<UNode, UError>;

pub trait UNodeProjector {
    fn to_expr_result(self) -> Result<UntypedExpr, UError>;
    fn to_def_result(self) -> Result<UntypedDefinition, UError>;
    fn to_script_result(self) -> Result<Vec<UntypedDefinition>, UError>;
}

impl UNodeProjector for UNode {
    fn to_expr_result(self) -> Result<UntypedExpr, UError> {
        match self {
            UNode::Expr(v) => Ok(v),
            _ => Err(UError::ExpectingExpr),
        }
    }

    fn to_def_result(self) -> Result<UntypedDefinition, UError> {
        match self {
            UNode::Def(v) => Ok(v),
            _ => Err(UError::ExpectingDefinition),
        }
    }

    fn to_script_result(self) -> Result<Vec<UntypedDefinition>, UError> {
        match self {
            UNode::Script(v) => Ok(v),
            _ => Err(UError::ExpectingScript),
        }
    }
}

impl IRVisitor<UResult> for ModuleBuilderFromIR {
    fn new() -> Self {
        ModuleBuilderFromIR::default()
    }

    fn visit_script(&self, script: &IRScript) -> UResult {
        // We allow only function definitions at the top level.
        // Why? Well the translation of IR to a random backend, such as Aiken's UntypedDeclaration
        // and UntypedExpr nodes, is by necessity tied to the semantics of the backend.
        // Also, we just need to demonstrate feasibility at the moment, not completeness or even
        // correctness.
        let mut fundefs = Vec::with_capacity(script.body.len());
        for ir in &script.body {
            let fundef = match ir {
                IR::FunDef(v) => Ok(v),
                _ => Err(UError::ExpectingFunDef),
            }?;

            let fundef = self.visit_fundef(fundef)?.to_def_result()?;
            fundefs.push(fundef);
        }

        Ok(fundefs.into())
    }

    fn visit_fundef(&self, fundef: &IRFunDef) -> UResult {
        let mut arguments = Vec::with_capacity(fundef.params.len());
        for param in &fundef.params {
            let location = no_span();
            let name = param.name.as_ref().ident.clone();
            let arg_name = ArgName::Named { name, location };

            let location = no_span();
            let arg = Arg {
                arg_name,
                location: no_span(),
                annotation: None,
                tipo: (),
            };

            arguments.push(arg);
        }
        let body = fundef.body.as_ref();
        let body = self.visit_blockstmt(body)?.to_expr_result()?;
        let doc = None;
        let location = no_span();
        let name = fundef.name.as_ref().ident.clone();

        let result = UntypedDefinition::Fn(Function {
            arguments,
            body,
            doc,
            location,
            name,
            public: true,
            return_annotation: None,
            return_type: (),
            end_position: 0,
        });

        Ok(result.into())
    }

    fn visit_vardef(&self, vardef: &IRVarDef) -> UResult {
        let location = no_span();
        let name = vardef.name.as_ref().ident.clone();

        let value = self.visit_expr(vardef.value.as_ref())?.to_expr_result()?;
        let value = Box::new(value);

        let pattern = UntypedPattern::Var { location, name };

        let result = UntypedExpr::Assignment {
            location,
            value,
            pattern,
            kind: AssignmentKind::Let,
            annotation: None,
        };

        Ok(result.into())
    }

    fn visit_retstmt(&self, retstmt: &IRReturnStmt) -> UResult {
        // TODO the semantics are not quite right here
        //      but this can be a good enough approximation for a demo.
        match &retstmt.expr {
            None => Err(UError::ExpectingExpr),
            Some(v) => {
                let expr = self.visit_expr(v)?.to_expr_result()?;
                Ok(expr.into())
            }
        }
    }

    fn visit_blockstmt(&self, blockstmt: &IRBlockStmt) -> UResult {
        let location = no_span();
        let body = &blockstmt.body;
        let mut expressions = Vec::with_capacity(body.len());

        for ir in body {
            let expr = self.visit_ir(ir)?.to_expr_result()?;
            expressions.push(expr);
        }

        let result = UntypedExpr::Sequence {
            location: no_span(),
            expressions,
        };

        Ok(result.into())
    }

    fn visit_ifstmt(&self, ifstmt: &IRIfStmt) -> UResult {
        let location = no_span();

        let condition = self.visit_expr(ifstmt._if.as_ref())?.to_expr_result()?;
        let body = self.visit_ir(ifstmt._else.as_ref())?.to_expr_result()?;

        let if_branch = IfBranch {
            condition,
            body,
            location,
        };
        let branches = vec1![if_branch];

        let final_else = self.visit_ir(ifstmt._else.as_ref())?.to_expr_result()?;
        let final_else = Box::new(final_else);

        let result = UntypedExpr::If {
            location: no_span(),
            branches,
            final_else,
        };

        Ok(result.into())
    }

    fn visit_exprstmt(&self, exprstmt: &IRExprStmt) -> UResult {
        let expr = self.visit_expr(&exprstmt.expr)?.to_expr_result()?;

        let result = UntypedExpr::Sequence {
            location: no_span(),
            expressions: vec![expr],
        };

        Ok(result.into())
    }

    fn visit_literal(&self, literal: &IRLiteral) -> UResult {
        let location = no_span();

        let result = match literal {
            IRLiteral::Boolean(v) => UntypedExpr::Var {
                location,
                name: (if *v { "True" } else { "False" }).to_string(),
            },
            IRLiteral::Float64(v) => UntypedExpr::Int {
                location,
                value: v.to_string(), // TODO IRLiteral::Float64 to UntypedExpr::Int ??
            },
            IRLiteral::String(v) => UntypedExpr::String {
                location,
                value: v.clone(),
            },
            IRLiteral::BigInt(v) => UntypedExpr::Int {
                location,
                value: v.to_string(),
            },
        };

        Ok(result.into())
    }

    fn visit_ident(&self, ident: &IRIdent) -> UResult {
        let result = UntypedExpr::Var {
            location: no_span(),
            name: ident.ident.clone(),
        };

        Ok(result.into())
    }

    fn visit_binary_op(&self, binary_op: &IRBinaryExpr) -> UResult {
        let location = no_span();
        let name = ir_bin_op_to_bin_op(binary_op.op.clone());

        let left = self.visit_expr(binary_op.left.as_ref())?.to_expr_result()?;
        let right = self
            .visit_expr(binary_op.right.as_ref())?
            .to_expr_result()?;

        let left = Box::new(left);
        let right = Box::new(right);

        let result = UntypedExpr::BinOp {
            location,
            name,
            left,
            right,
        };

        Ok(result.into())
    }

    fn visit_apply(&self, apply: &IRApply) -> UResult {
        // TODO Implement
        todo!()
    }
}
