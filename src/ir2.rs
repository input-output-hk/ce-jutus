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
    pub expr: IRExpr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IRReturnStmt {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct IRScript {
    pub body: Vec<IR>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IRDef {
    Fun(IRFunDef), // js: function add(a, b) { return a + b; }
    Var(IRVarDef), // js: let a = 1;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IRStmt {
    If(IRIfStmt), // js: if(cond) { if_stmt; } else { else_stmt; }
    Return(IRReturnStmt),
    Block(IRBlockStmt),
    Expr(IRExprStmt),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IR {
    Script(Box<IRScript>),
    Def(Box<IRDef>),
    Stmt(Box<IRStmt>),
    Expr(Box<IRExpr>),
}
