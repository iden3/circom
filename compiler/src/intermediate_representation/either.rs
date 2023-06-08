use program_structure::ast::{Expression, Statement};

#[derive(Clone, Debug)]
pub enum EitherExprOrStmt {
    Expr(Expression),
    Stmt(Statement)
}

impl EitherExprOrStmt {
    pub fn unwrap_expr(&self) -> &Expression {
        match self {
            EitherExprOrStmt::Expr(e) => e,
            EitherExprOrStmt::Stmt(_) => panic!("Attempted to unwrap a expression in a Statement variant")
        }
    }

    pub fn unwrap_stmt(&self) -> &Statement {
        match self {
            EitherExprOrStmt::Expr(_) => panic!("Attempted to unwrap a statement in a Expression variant"),
            EitherExprOrStmt::Stmt(s) => s
        }
    }

    pub fn from_stmt(stmt: Statement) -> Self {
        EitherExprOrStmt::Stmt(stmt)
    }

    pub fn from_expr(expr: Expression) -> Self {
        EitherExprOrStmt::Expr(expr)
    }
}