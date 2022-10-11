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

pub fn build_log_call(meta: Meta, args: Vec<LogArgument>) -> Statement {
    let mut new_args = Vec::new();
    for arg in args {
        match arg {
            LogArgument::LogExp(..) => { new_args.push(arg);}
            LogArgument::LogStr(str) => { new_args.append(&mut split_string(str));}
        }
    }
    LogCall { meta, args: new_args }
}

fn split_string(str: String) -> Vec<LogArgument> {
    let mut v = vec![];
    let sub_len = 230;
    let mut cur = str;
    while !cur.is_empty() {
        let (chunk, rest) = cur.split_at(std::cmp::min(sub_len, cur.len()));
        v.push(LogArgument::LogStr(chunk.to_string()));
        cur = rest.to_string();
    }
    v
}

pub fn build_assert(meta: Meta, arg: Expression) -> Statement {
    Assert { meta, arg }
}

pub fn build_mult_substitution(meta: Meta, lhe: Expression, op : AssignOp, rhe: Expression) -> Statement {
    MultSubstitution { meta: meta.clone(), lhe, op, rhe }
}

pub fn build_anonymous_component_statement(meta: Meta, arg: Expression) -> Statement {
    MultSubstitution { meta: meta.clone(), lhe: crate::expression_builders::build_tuple(meta, Vec::new()), op: AssignOp::AssignConstraintSignal, rhe: arg }
}