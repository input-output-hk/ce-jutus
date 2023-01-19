use std::fmt::Debug;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use ir_type::IRTy;

use crate::ir_type;
use crate::ir_type::{IRFunTy, TypeOf};

#[derive(Debug, Serialize, Deserialize)]
pub struct IRIdent {
    pub ident: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IRLiteral {
    Boolean(bool),
    Float64(f64),
    BigInt(BigInt),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRNameTy {
    #[serde(flatten)]
    pub name: Box<IRIdent>,
    pub ty: IRTy,
}

impl TypeOf for IRNameTy {
    fn type_of(&self) -> IRTy {
        self.ty.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRFunDef {
    #[serde(flatten)]
    pub name: Box<IRIdent>,
    pub params: Vec<IRNameTy>,
    pub fun_ty: IRFunTy,
    #[serde(flatten)]
    pub body: Box<IRBlockStmt>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRVarDef {
    #[serde(flatten)]
    pub name: Box<IRIdent>,
    pub ty: IRTy,
    pub is_mutable: bool,
    pub value: Box<IRExpr>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IRBinOp {
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,

    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,

    /// `||`
    LogicalOr,

    /// `&&`
    LogicalAnd,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRBinaryExpr {
    pub op: IRBinOp,
    pub left: Box<IRExpr>,
    pub right: Box<IRExpr>,
}

/// Function application
#[derive(Debug, Serialize, Deserialize)]
pub struct IRApply {
    #[serde(flatten)]
    pub name: Box<IRIdent>,
    pub args: Vec<IRExpr>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRIfStmt {
    #[serde(rename = "if")]
    pub _if: Box<IRExpr>,
    #[serde(rename = "then")]
    pub _then: Box<IR>,
    #[serde(rename = "else")]
    pub _else: Box<IR>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRExprStmt {
    #[serde(flatten)]
    pub expr: IRExpr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRReturnStmt {
    #[serde(flatten)]
    pub expr: Option<IRExpr>,
}
/// Expressions
#[derive(Debug, Serialize, Deserialize)]
pub enum IRExpr {
    Literal(IRLiteral),
    Identifier(IRIdent),
    Binary(Box<IRBinaryExpr>),
    Apply(Box<IRApply>),
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IRBlockStmt {
    pub body: Vec<IR>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IRScript {
    pub body: Vec<IR>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IR {
    /// A program, module, etc.
    Script(IRScript),
    /// `function` definition
    FunDef(IRFunDef),
    /// variable definition
    VarDef(IRVarDef),
    /// `return` statement
    ReturnStmt(IRReturnStmt),
    /// Block of statements and expressions
    BlockStmt(IRBlockStmt),
    /// `if`/`then`/`else` statement
    IfStmt(IRIfStmt),
    ///
    ExprStmt(IRExprStmt),
    /// Expression
    Expr(IRExpr),
    /// Parenthesized expression
    Paren(IRExpr),
}
