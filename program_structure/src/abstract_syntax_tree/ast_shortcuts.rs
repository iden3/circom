use super::ast::*;
use super::expression_builders::*;
use super::statement_builders::*;
use crate::ast::{Access, Expression, VariableType};
use num_bigint::BigInt;

#[derive(Clone)]
pub struct Symbol {
    pub name: String,
    pub is_array: Vec<Expression>,
    pub init: Option<Expression>,
}

pub fn assign_with_op_shortcut(
    op: ExpressionInfixOpcode,
    meta: Meta,
    variable: (String, Vec<Access>),
    rhe: Expression,
) -> Statement {
    let (var, access) = variable;
    let variable = build_variable(meta.clone(), var.clone(), access.clone());
    let infix = build_infix(meta.clone(), variable, op, rhe);
    build_substitution(meta, var, access, AssignOp::AssignVar, infix)
}

pub fn plusplus(meta: Meta, variable: (String, Vec<Access>)) -> Statement {
    let one = build_number(meta.clone(), BigInt::from(1));
    assign_with_op_shortcut(ExpressionInfixOpcode::Add, meta, variable, one)
}

pub fn subsub(meta: Meta, variable: (String, Vec<Access>)) -> Statement {
    let one = build_number(meta.clone(), BigInt::from(1));
    assign_with_op_shortcut(ExpressionInfixOpcode::Sub, meta, variable, one)
}

pub fn for_into_while(
    meta: Meta,
    init: Statement,
    cond: Expression,
    step: Statement,
    body: Statement,
) -> Statement {
    let while_body = build_block(body.get_meta().clone(), vec![body, step]);
    let while_statement = build_while_block(meta.clone(), cond, while_body);
    build_block(meta, vec![init, while_statement])
}

pub fn split_declaration_into_single_nodes(
    meta: Meta,
    xtype: VariableType,
    symbols: Vec<Symbol>,
) -> Statement {
    let mut initializations = Vec::new();

    for symbol in symbols {
        let with_meta = meta.clone();
        let has_type = xtype;
        let name = symbol.name.clone();
        let dimensions = symbol.is_array;
        let possible_init = symbol.init;
        let single_declaration = build_declaration(with_meta, has_type, name, dimensions);
        initializations.push(single_declaration);

        if let Option::Some(init) = possible_init {
            let substitution =
                build_substitution(meta.clone(), symbol.name, vec![], AssignOp::AssignVar, init);
            initializations.push(substitution);
        }
    }
    build_initialization_block(meta, xtype, initializations)
}
