use crate::ir::*;

pub trait IRVisitor<VResult> {
    fn new() -> Self;

    fn run(&self, ir: &IR) -> VResult {
        self.visit_ir(ir)
    }

    fn visit_ir(&self, ir: &IR) -> VResult {
        match ir {
            IR::Script(script) => self.visit_script(script),
            IR::FunDef(fundef) => self.visit_fundef(fundef),
            IR::VarDef(vardef) => self.visit_vardef(vardef),
            IR::ReturnStmt(retstmt) => self.visit_retstmt(retstmt),
            IR::BlockStmt(blockstmt) => self.visit_blockstmt(blockstmt),
            IR::IfStmt(ifstmt) => self.visit_ifstmt(ifstmt),
            IR::ExprStmt(exprstmt) => self.visit_exprstmt(exprstmt),
            IR::Expr(expr) => self.visit_expr(expr),
            IR::Paren(expr) => self.visit_expr(expr),
        }
    }

    fn visit_script(&self, script: &IRScript) -> VResult;
    fn visit_fundef(&self, fundef: &IRFunDef) -> VResult;
    fn visit_vardef(&self, vardef: &IRVarDef) -> VResult;
    fn visit_retstmt(&self, retstmt: &IRReturnStmt) -> VResult;
    fn visit_blockstmt(&self, blockstmt: &IRBlockStmt) -> VResult;
    fn visit_ifstmt(&self, ifstmt: &IRIfStmt) -> VResult;
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
