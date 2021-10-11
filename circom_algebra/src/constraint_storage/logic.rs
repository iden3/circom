use super::*;
use std::collections::HashMap;

pub fn code_expression(expr: HashMap<S, BigInt>, tracker: &mut FieldTracker) -> CompressedExpr {
    let mut c_expr = CompressedExpr::new();
    for (var, coeff) in expr {
        let raw_coeff = coeff.to_signed_bytes_le();
        let coeff_id = tracker.insert(raw_coeff);
        c_expr.push((coeff_id, var));
    }
    c_expr
}

pub fn code_constraint(constraint: C, tracker: &mut FieldTracker) -> CompressedConstraint {
    let a = code_expression(constraint.a, tracker);
    let b = code_expression(constraint.b, tracker);
    let c = code_expression(constraint.c, tracker);
    (a, b, c)
}

pub fn decode_expr(c_expr: &CompressedExpr, tracker: &FieldTracker) -> HashMap<S, BigInt> {
    let mut decoded_expr = HashMap::new();
    for (coeff_id, var) in c_expr {
        let raw_coeff = tracker.get_constant(*coeff_id).unwrap();
        let coeff = BigInt::from_signed_bytes_le(raw_coeff);
        decoded_expr.insert(*var, coeff);
    }
    decoded_expr
}

pub fn decode_constraint(constraint: &CompressedConstraint, tracker: &FieldTracker) -> C {
    let (a, b, c) = constraint;
    C { a: decode_expr(a, tracker), b: decode_expr(b, tracker), c: decode_expr(c, tracker) }
}
