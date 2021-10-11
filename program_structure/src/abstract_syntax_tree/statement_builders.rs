use super::ast::*;
use Statement::*;

pub fn build_conditional_block(
    meta: Meta,
    cond: Expression,
    if_case: Statement,
    else_case: Option<Statement>,
) -> Statement {
    IfThenElse { meta, cond, else_case: else_case.map(|s| Box::new(s)), if_case: Box::new(if_case) }
}

pub fn build_while_block(meta: Meta, cond: Expression, stmt: Statement) -> Statement {
    While { meta, cond, stmt: Box::new(stmt) }
}

pub fn build_initialization_block(
    meta: Meta,
    xtype: VariableType,
    initializations: Vec<Statement>,
) -> Statement {
    InitializationBlock { meta, xtype, initializations }
}
pub fn build_block(meta: Meta, stmts: Vec<Statement>) -> Statement {
    Block { meta, stmts }
}

pub fn build_return(meta: Meta, value: Expression) -> Statement {
    Return { meta, value }
}

pub fn build_declaration(
    meta: Meta,
    xtype: VariableType,
    name: String,
    dimensions: Vec<Expression>,
) -> Statement {
    let is_constant = true;
    Declaration { meta, xtype, name, dimensions, is_constant }
}

pub fn build_substitution(
    meta: Meta,
    var: String,
    access: Vec<Access>,
    op: AssignOp,
    rhe: Expression,
) -> Statement {
    Substitution { meta, var, access, op, rhe }
}

pub fn build_constraint_equality(meta: Meta, lhe: Expression, rhe: Expression) -> Statement {
    ConstraintEquality { meta, lhe, rhe }
}

pub fn build_log_call(meta: Meta, arg: Expression) -> Statement {
    LogCall { meta, arg }
}

pub fn build_assert(meta: Meta, arg: Expression) -> Statement {
    Assert { meta, arg }
}
