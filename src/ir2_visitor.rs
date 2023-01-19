use crate::ir2::*;

pub trait IR2Visitor<VResult> {
    fn new() -> Self;

    fn run(&self, ir: &IR) -> VResult {
        self.visit_ir(ir)
    }

    fn visit_ir(&self, ir: &IR) -> VResult {
        match ir {
            IR::Script(v) => self.visit_module(v.as_ref()),
            IR::Def(v) => self.visit_def(v.as_ref()),
            IR::Stmt(v) => self.visit_stmt(v.as_ref()),
            IR::Expr(v) => self.visit_expr(v),
        }
    }

    fn visit_module(&self, module: &IRScript) -> VResult;

    fn visit_def(&self, def: &IRDef) -> VResult {
        match def {
            IRDef::Fun(v) => self.visit_fundef(v),
            IRDef::Var(v) => self.visit_vardef(v),
        }
    }

    fn visit_fundef(&self, fundef: &IRFunDef) -> VResult;
    fn visit_vardef(&self, vardef: &IRVarDef) -> VResult;

    fn visit_stmt(&self, stmt: &IRStmt) -> VResult {
        match stmt {
            IRStmt::If(v) => self.visit_ifstmt(v),
            IRStmt::Return(v) => self.visit_retstmt(v),
            IRStmt::Block(v) => self.visit_blockstmt(v),
            IRStmt::Expr(v) => self.visit_exprstmt(v),
        }
    }

    fn visit_ifstmt(&self, ifstmt: &IRIfStmt) -> VResult;
    fn visit_retstmt(&self, retstmt: &IRReturnStmt) -> VResult;
    fn visit_blockstmt(&self, blockstmt: &IRBlockStmt) -> VResult;
    fn visit_exprstmt(&self, exprstmt: &IRExprStmt) -> VResult;

    fn visit_expr(&self, expr: &IRExpr) -> VResult {
        match expr {
            IRExpr::Literal(literal) => self.visit_literal(literal),
            IRExpr::Identifier(ident) => self.visit_ident(ident), // todo this needs a symbol table?
            IRExpr::Binary(binary_op) => self.visit_binary_op(binary_op),
            IRExpr::Apply(apply) => self.visit_apply(apply),
        }
    }

    fn visit_literal(&self, literal: &IRLiteral) -> VResult;
    fn visit_ident(&self, ident: &IRIdent) -> VResult;
    fn visit_binary_op(&self, binary_op: &IRBinaryExpr) -> VResult;
    fn visit_apply(&self, apply: &IRApply) -> VResult;
}
