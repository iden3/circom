use super::ast::*;
use num_bigint::BigInt;
use Expression::*;

pub fn build_infix(
    meta: Meta,
    lhe: Expression,
    infix_op: ExpressionInfixOpcode,
    rhe: Expression,
) -> Expression {
    InfixOp { meta, infix_op, lhe: Box::new(lhe), rhe: Box::new(rhe) }
}

pub fn build_prefix(meta: Meta, prefix_op: ExpressionPrefixOpcode, rhe: Expression) -> Expression {
    PrefixOp { meta, prefix_op, rhe: Box::new(rhe) }
}

pub fn build_inline_switch_op(
    meta: Meta,
    cond: Expression,
    if_true: Expression,
    if_false: Expression,
) -> Expression {
    InlineSwitchOp {
        meta,
        cond: Box::new(cond),
        if_true: Box::new(if_true),
        if_false: Box::new(if_false),
    }
}

pub fn build_variable(meta: Meta, name: String, access: Vec<Access>) -> Expression {
    Variable { meta, name, access }
}

pub fn build_number(meta: Meta, value: BigInt) -> Expression {
    Expression::Number(meta, value)
}

pub fn build_call(meta: Meta, id: String, args: Vec<Expression>) -> Expression {
    Call { meta, id, args }
}

pub fn build_array_in_line(meta: Meta, values: Vec<Expression>) -> Expression {
    ArrayInLine { meta, values }
}
