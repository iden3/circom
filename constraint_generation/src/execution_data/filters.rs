use super::analysis::Analysis;
use program_structure::ast::*;

pub fn apply_unused(stmt: &mut Statement, analysis: &Analysis, prime: &String) {
    clean_dead_code(stmt, analysis, prime);
}

fn clean_dead_code(stmt: &mut Statement, analysis: &Analysis, prime: &String) -> bool {
    use circom_algebra::modular_arithmetic::as_bool;
    use Statement::*;
    match stmt {
        While { stmt, .. } => clean_dead_code(stmt, analysis, prime),
        MultSubstitution { .. } => unreachable!(),
        IfThenElse { if_case, else_case, cond, meta } => {
            let field = program_structure::constants::UsefulConstants::new(prime).get_p().clone();
            let empty_block = Box::new(Block { meta: meta.clone(), stmts: vec![] });
            let if_case_empty = clean_dead_code(if_case, analysis, prime);
            let else_case_empty =
                if let Some(case) = else_case { clean_dead_code(case, analysis, prime) } else { true };
            if else_case_empty {
                *else_case = None;
            }

            match Analysis::read_computed(analysis, cond.get_meta().elem_id) {
                Some(val) if as_bool(&val, &field) => *stmt = *if_case.clone(),
                Some(val) if !as_bool(&val, &field) => {
                    *stmt = *else_case.clone().unwrap_or(empty_block)
                }
                _ => {}
            }
            if_case_empty && else_case_empty
        }
        Block { stmts, .. } => {
            let work = std::mem::take(stmts);
            for mut w in work {
                let id = w.get_meta().elem_id;
                if Analysis::is_reached(analysis, id) {
                    let empty = clean_dead_code(&mut w, analysis, prime);
                    if !empty {
                        stmts.push(w)
                    }
                }
            }
            stmts.is_empty()
        }
        _ => false,
    }
}

pub fn apply_computed(stmt: &mut Statement, analysis: &Analysis) {
    use Statement::*;
    match stmt {
        MultSubstitution { .. } => unreachable!(),
        IfThenElse { cond, if_case, else_case, .. } => {
            *cond = computed_or_original(analysis, cond);
            apply_computed_expr(cond, analysis);
            apply_computed(if_case, analysis);
            if let Some(stmt) = else_case {
                apply_computed(stmt, analysis);
            }
        }
        While { cond, stmt, .. } => {
            *cond = computed_or_original(analysis, cond);
            apply_computed_expr(cond, analysis);
            apply_computed(stmt, analysis);
        }
        Block { stmts, .. } => {
            apply_computed_stmt_vec(stmts, analysis);
        }
        InitializationBlock { initializations, .. } => {
            apply_computed_stmt_vec(initializations, analysis);
        }
        Return { value, .. } => {
            *value = computed_or_original(analysis, value);
            apply_computed_expr(value, analysis);
        }
        Declaration { dimensions, .. } => {
            apply_computed_expr_vec(dimensions, analysis);
        }
        Substitution { access, rhe, .. } => {
            *rhe = computed_or_original(analysis, rhe);
            apply_computed_expr(rhe, analysis);
            apply_computed_to_access(access, analysis);
        }
        ConstraintEquality { lhe, rhe, .. } => {
            *lhe = computed_or_original(analysis, lhe);
            *rhe = computed_or_original(analysis, rhe);
            apply_computed_expr(lhe, analysis);
            apply_computed_expr(rhe, analysis);
        }
        LogCall { args, .. } => {
            for arglog in args {
                if let LogArgument::LogExp(arg) = arglog{
                    *arg = computed_or_original(analysis, arg);
                    apply_computed_expr(arg, analysis);
                }
            }
        }
        Assert { arg, .. } => {
            *arg = computed_or_original(analysis, arg);
            apply_computed_expr(arg, analysis);
        }
    }
}

fn apply_computed_stmt_vec(stmts: &mut Vec<Statement>, analysis: &Analysis) {
    for stmt in stmts {
        apply_computed(stmt, analysis);
    }
}

fn apply_computed_to_access(accesses: &mut Vec<Access>, analysis: &Analysis) {
    use Access::*;
    let work = std::mem::take(accesses);
    for acc in work {
        match acc {
            ArrayAccess(mut index) => {
                index = computed_or_original(analysis, &index);
                apply_computed_expr(&mut index, analysis);
                accesses.push(Access::ArrayAccess(index));
            }
            _ => {
                accesses.push(acc);
            }
        }
    }
}

fn apply_computed_expr_vec(exprs: &mut Vec<Expression>, analysis: &Analysis) {
    let work = std::mem::take(exprs);
    for w in work {
        let mut expr = computed_or_original(analysis, &w);
        apply_computed_expr(&mut expr, analysis);
        exprs.push(expr);
    }
}

fn computed_or_original(analysis: &Analysis, expr: &Expression) -> Expression {
    let id = expr.get_meta().elem_id;
    if let Some(val) = Analysis::read_computed(analysis, id) {
        Expression::Number(expr.get_meta().clone(), val)
    } else {
        expr.clone()
    }
}

fn apply_computed_expr(expr: &mut Expression, analysis: &Analysis) {
    use Expression::*;
    match expr {
        InfixOp { lhe, rhe, .. } => {
            *lhe = Box::new(computed_or_original(analysis, lhe));
            *rhe = Box::new(computed_or_original(analysis, rhe));
            apply_computed_expr(lhe, analysis);
            apply_computed_expr(rhe, analysis);
        }
        PrefixOp { rhe, .. } => {
            *rhe = Box::new(computed_or_original(analysis, rhe));
            apply_computed_expr(rhe, analysis);
        }
        InlineSwitchOp { if_true, if_false, cond, .. } => {
            *if_true = Box::new(computed_or_original(analysis, if_true));
            *if_false = Box::new(computed_or_original(analysis, if_false));
            *cond = Box::new(computed_or_original(analysis, cond));
            apply_computed_expr(if_true, analysis);
            apply_computed_expr(if_false, analysis);
            apply_computed_expr(cond, analysis);
        }
        Variable { access, .. } => {
            apply_computed_to_access(access, analysis);
        }
        Number(..) => {}
        Call { args, .. } => {
            apply_computed_expr_vec(args, analysis);
        }
        ArrayInLine { values, .. } => {
            apply_computed_expr_vec(values, analysis);
        }
        UniformArray { value, dimension, .. } => {
            *value = Box::new(computed_or_original(analysis, value));
            *dimension = Box::new(computed_or_original(analysis, dimension));
            apply_computed_expr(value, analysis);
            apply_computed_expr(dimension, analysis);
        }
        ParallelOp {rhe, .. } => {
            *rhe = Box::new(computed_or_original(analysis, rhe));
            apply_computed_expr(rhe, analysis);
        }
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}
