use std::fmt::Debug;

use std::path::PathBuf;
use std::rc::Rc;

use thiserror::Error;

use crate::ir::IR;
use crate::ir_to_unode::UError;
use crate::js_compiler::JsError;
use crate::js_to_ir::IRError;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Language {
    /// Javascript, Typescript, whatever [`swc`](https://github.com/swc-project/swc) recognizes
    Javascript,
    /// Because why not. Maybe use <https://github.com/RustPython/RustPython/> ?
    Python,
    /// Look, ma, I got Smart Contracts! Maybe use <https://github.com/hyperledger/solang> ?
    Solidity,
    /// See <https://tech.mystenlabs.com/why-we-created-sui-move/>
    Move,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source<X> {
    pub lang: Language,
    pub extra: X,
    pub script_name: String,
    pub script_path: PathBuf,
    pub code: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while parsing")]
    Js(JsError),
    #[error("Error while generating IR from swc ASTs")]
    IR(IRError),
    #[error("Error compiling to aiken untyped trees")]
    U(UError),
    #[error("Aiken error")]
    Aiken(aiken_project::error::Error),
    #[error("Bad file name")]
    BadFilename(PathBuf),
}

#[derive(Debug)]
pub struct Program<X> {
    pub source: Rc<Source<X>>,
    pub ir: IR,
}
