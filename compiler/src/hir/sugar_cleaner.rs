use super::very_concrete_program::*;
use program_structure::ast::*;
use num_traits::{ToPrimitive};


struct ExtendedSyntax {
    initializations: Vec<Statement>,
}

struct Context {}

struct State {
    fresh_id: usize,
}
impl State {
    pub fn produce_id(&mut self) -> String {
        let fresh = self.fresh_id;
        self.fresh_id += 1;
        format!("{}_auto", fresh)
    }
}

/*
    Tasks fulfilled by this algorithm:
        -Nested call removal
        -Inline if-then-else removal
        -Inline array removal
        -Initialization Block removal (no longer needed)
        -Uniform array removal
*/

pub fn clean_sugar(vcp: &mut VCP) {
    let mut state = State { fresh_id: 0 };
    for template in &mut vcp.templates {
        let context = Context {};
        let trash = extend_statement(&mut template.code, &mut state, &context);
        assert!(trash.is_empty());
    }
    for vcf in &mut vcp.functions {
        let context = Context {};
        let trash = extend_statement(&mut vcf.body, &mut state, &context);
        assert!(trash.is_empty());
    }
}

fn extend_statement(stmt: &mut Statement, state: &mut State, context: &Context) -> Vec<Statement> {
    if stmt.is_block() {
        extend_block(stmt, state, context)
    } else if stmt.is_while() {
        extend_while(stmt, state, context)
    } else if stmt.is_if_then_else() {
        extend_conditional(stmt, state, context)
    } else if stmt.is_substitution() {
        extend_substitution(stmt, state, context)
    } else if stmt.is_constraint_equality() {
        extend_constraint_equality(stmt, state, context)
    } else if stmt.is_declaration() {
        extend_declaration(stmt, state, context)
    } else if stmt.is_return() {
        extend_return(stmt, state, context)
    } else if stmt.is_log_call() {
        extend_log_call(stmt, state, context)
    } else if stmt.is_assert() {
        extend_assert(stmt, state, context)
    } else {
        unreachable!()
    }
}

fn extend_declaration(
    _stmt: &mut Statement,
    _state: &mut State,
    _context: &Context,
) -> Vec<Statement> {
    vec![]
}

fn extend_block(stmt: &mut Statement, state: &mut State, context: &Context) -> Vec<Statement> {
    use Statement::Block;
    if let Block { stmts, .. } = stmt {
        let checkpoint_id = state.fresh_id;
        map_init_blocks(stmts);
        map_stmts_with_sugar(stmts, state, context);
        map_substitutions(stmts);
        map_constraint_equalities(stmts);
        state.fresh_id = map_returns(stmts, state.fresh_id);
        state.fresh_id = checkpoint_id;
        vec![]
    } else {
        unreachable!()
    }
}

fn extend_while(stmt: &mut Statement, state: &mut State, context: &Context) -> Vec<Statement> {
    use Statement::While;
    if let While { cond, stmt, .. } = stmt {
        let mut expands = extend_statement(stmt, state, context);
        let mut cond_extension = extend_expression(cond, state, context);
        expands.append(&mut cond_extension.initializations);
        let mut expr = vec![cond.clone()];
        sugar_filter(&mut expr, state, &mut expands);
        *cond = expr.pop().unwrap();
        expands
    } else {
        unreachable!()
    }
}

fn extend_conditional(
    stmt: &mut Statement,
    state: &mut State,
    context: &Context,
) -> Vec<Statement> {
    use Statement::IfThenElse;
    if let IfThenElse { cond, if_case, else_case, .. } = stmt {
        let mut expands = vec![];
        expands.append(&mut extend_statement(if_case, state, context));
        if let Option::Some(s) = else_case {
            expands.append(&mut extend_statement(s, state, context));
        }
        let mut cond_extension = extend_expression(cond, state, context);
        expands.append(&mut cond_extension.initializations);
        let mut expr = vec![cond.clone()];
        sugar_filter(&mut expr, state, &mut expands);
        *cond = expr.pop().unwrap();
        expands
    } else {
        unreachable!()
    }
}

fn extend_substitution(
    stmt: &mut Statement,
    state: &mut State,
    context: &Context,
) -> Vec<Statement> {
    use Statement::Substitution;
    if let Substitution { rhe, .. } = stmt {
        extend_expression(rhe, state, context).initializations
    } else {
        unreachable!()
    }
}

fn extend_constraint_equality(
    stmt: &mut Statement,
    state: &mut State,
    context: &Context,
) -> Vec<Statement> {
    use Statement::ConstraintEquality;
    if let ConstraintEquality { lhe, rhe, .. } = stmt {
        let mut expands = extend_expression(lhe, state, context).initializations;
        expands.append(&mut extend_expression(rhe, state, context).initializations);
        expands
    } else {
        unreachable!()
    }
}

fn extend_return(stmt: &mut Statement, state: &mut State, context: &Context) -> Vec<Statement> {
    use Statement::Return;
    if let Return { value, .. } = stmt {
        extend_expression(value, state, context).initializations
    } else {
        unreachable!()
    }
}

fn extend_log_call(stmt: &mut Statement, state: &mut State, context: &Context) -> Vec<Statement> {
    use Statement::LogCall;
    if let LogCall { args, .. } = stmt {
        let mut initializations = Vec::new();
        for arglog in args {
            if let LogArgument::LogExp(arg) = arglog {
                let mut exp = extend_expression(arg, state, context);
                initializations.append(&mut exp.initializations);
            }
        }
        initializations
    } else {
        unreachable!()
    }
}

fn extend_assert(stmt: &mut Statement, state: &mut State, context: &Context) -> Vec<Statement> {
    use Statement::Assert;
    if let Assert { arg, .. } = stmt {
        extend_expression(arg, state, context).initializations
    } else {
        unreachable!()
    }
}

fn extend_expression(
    expr: &mut Expression,
    state: &mut State,
    context: &Context,
) -> ExtendedSyntax {
    if expr.is_number() {
        extend_number(expr, state, context)
    } else if expr.is_variable() {
        extend_variable(expr, state, context)
    } else if expr.is_array() {
        extend_array(expr, state, context)
    } else if expr.is_call() {
        extend_call(expr, state, context)
    } else if expr.is_prefix() {
        extend_prefix(expr, state, context)
    } else if expr.is_infix() {
        extend_infix(expr, state, context)
    } else if expr.is_switch() {
        extend_switch(expr, state, context)
    } else if expr.is_parallel(){
        extend_parallel(expr, state, context)
    }else {
        unreachable!()
    }
}

fn extend_number(_expr: &mut Expression, _state: &mut State, _context: &Context) -> ExtendedSyntax {
    ExtendedSyntax { initializations: vec![] }
}

fn extend_variable(expr: &mut Expression, state: &mut State, context: &Context) -> ExtendedSyntax {
    use Expression::Variable;
    if let Variable { access, .. } = expr {
        let mut inits = vec![];
        for acc in access {
            if let Access::ArrayAccess(e) = acc {
                let mut expand = extend_expression(e, state, context);
                inits.append(&mut expand.initializations);
                let mut expr = vec![e.clone()];
                sugar_filter(&mut expr, state, &mut inits);
                *e = expr.pop().unwrap();
            }
        }
        ExtendedSyntax { initializations: inits }
    } else {
        unreachable!();
    }
}

fn extend_prefix(expr: &mut Expression, state: &mut State, context: &Context) -> ExtendedSyntax {
    use Expression::PrefixOp;
    if let PrefixOp { rhe, .. } = expr {
        let mut extended = extend_expression(rhe, state, context);
        let mut expr = vec![*rhe.clone()];
        sugar_filter(&mut expr, state, &mut extended.initializations);
        *rhe = Box::new(expr.pop().unwrap());
        extended
    } else {
        unreachable!()
    }
}

fn extend_parallel(expr: &mut Expression, state: &mut State, context: &Context) -> ExtendedSyntax {
    use Expression::ParallelOp;
    if let ParallelOp { rhe, .. } = expr {
        let mut extended = extend_expression(rhe, state, context);
        let mut expr = vec![*rhe.clone()];
        sugar_filter(&mut expr, state, &mut extended.initializations);
        *rhe = Box::new(expr.pop().unwrap());
        extended
    } else {
        unreachable!()
    }
}

fn extend_infix(expr: &mut Expression, state: &mut State, context: &Context) -> ExtendedSyntax {
    use Expression::InfixOp;
    if let InfixOp { lhe, rhe, .. } = expr {
        let mut lh_expand = extend_expression(lhe, state, context);
        let mut rh_expand = extend_expression(rhe, state, context);
        lh_expand.initializations.append(&mut rh_expand.initializations);
        let mut extended = lh_expand;
        let mut expr = vec![*lhe.clone(), *rhe.clone()];
        sugar_filter(&mut expr, state, &mut extended.initializations);
        *rhe = Box::new(expr.pop().unwrap());
        *lhe = Box::new(expr.pop().unwrap());
        extended
    } else {
        unreachable!()
    }
}

fn extend_switch(expr: &mut Expression, state: &mut State, context: &Context) -> ExtendedSyntax {
    use Expression::InlineSwitchOp;
    if let InlineSwitchOp { if_true, if_false, .. } = expr {
        let mut true_expand = extend_expression(if_true, state, context);
        let mut false_expand = extend_expression(if_false, state, context);
        true_expand.initializations.append(&mut false_expand.initializations);
        let mut extended = true_expand;
        let mut expr = vec![*if_true.clone(), *if_false.clone()];
        sugar_filter(&mut expr, state, &mut extended.initializations);
        *if_false = Box::new(expr.pop().unwrap());
        *if_true = Box::new(expr.pop().unwrap());
        extended
    } else {
        unreachable!()
    }
}

fn extend_array(expr: &mut Expression, state: &mut State, context: &Context) -> ExtendedSyntax {
    use Expression::{ArrayInLine, UniformArray};
    if let ArrayInLine { values, .. } = expr {
        let mut initializations = vec![];
        for v in values.iter_mut() {
            let mut extended = extend_expression(v, state, context);
            initializations.append(&mut extended.initializations);
        }
        sugar_filter(values, state, &mut initializations);
        ExtendedSyntax { initializations }
    } else if let UniformArray { value, dimension, .. } = expr {
        let mut value_expand = extend_expression(value, state, context);
        let mut dimension_expand = extend_expression(dimension, state, context);
        value_expand.initializations.append(&mut dimension_expand.initializations);
        let mut extended = value_expand;
        let mut expr = vec![*value.clone(), *dimension.clone()];
        sugar_filter(&mut expr, state, &mut extended.initializations);
        *dimension = Box::new(expr.pop().unwrap());
        *value = Box::new(expr.pop().unwrap());
        extended
    } else {
        unreachable!();
    }
}

fn extend_call(expr: &mut Expression, state: &mut State, context: &Context) -> ExtendedSyntax {
    use Expression::Call;
    if let Call { args, .. } = expr {
        let mut inits = vec![];
        for a in args.iter_mut() {
            let mut extended = extend_expression(a, state, context);
            inits.append(&mut extended.initializations);
        }
        sugar_filter(args, state, &mut inits);
        ExtendedSyntax { initializations: inits }
    } else {
        unreachable!()
    }
}

// Utils

fn sugar_filter(elements: &mut Vec<Expression>, state: &mut State, inits: &mut Vec<Statement>) {
    let work = std::mem::replace(elements, Vec::with_capacity(elements.len()));
    for elem in work {
        let should_be_extracted = elem.is_call() || elem.is_switch() || elem.is_array();
        if should_be_extracted {
            let id = state.produce_id();
            let expr = rmv_sugar(&id, elem, inits);
            elements.push(expr);
        } else {
            elements.push(elem);
        }
    }
}

fn rmv_sugar(fresh_variable: &str, expr: Expression, buffer: &mut Vec<Statement>) -> Expression {
    use Expression::Variable;
    use Statement::{Declaration, Substitution};
    let mut declaration_meta = expr.get_meta().clone();
    let mut variable_meta = expr.get_meta().clone();
    let mut initialization_meta = expr.get_meta().clone();
    declaration_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
    variable_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
    initialization_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
    let declaration = Declaration {
        meta: declaration_meta,
        is_constant: true,
        xtype: VariableType::Var,
        name: fresh_variable.to_string(),
        dimensions: vec![],
    };
    let initialization = Substitution {
        meta: initialization_meta,
        var: fresh_variable.to_string(),
        access: vec![],
        op: AssignOp::AssignVar,
        rhe: expr,
    };
    let new_arg =
        Variable { meta: variable_meta, name: fresh_variable.to_string(), access: vec![] };
    buffer.push(declaration);
    buffer.push(initialization);
    new_arg
}

// mappings
fn map_stmts_with_sugar(stmts: &mut Vec<Statement>, state: &mut State, context: &Context) {
    let work = std::mem::take(stmts);
    for mut w in work {
        let mut inits = extend_statement(&mut w, state, context);
        stmts.append(&mut inits);
        stmts.push(w);
    }
}

fn map_init_blocks(stmts: &mut Vec<Statement>) {
    use Statement::InitializationBlock;
    let work = std::mem::take(stmts);
    for w in work {
        if let InitializationBlock { mut initializations, .. } = w {
            stmts.append(&mut initializations);
        } else {
            stmts.push(w);
        }
    }
}

fn map_substitutions(stmts: &mut Vec<Statement>) {
    let work = std::mem::take(stmts);
    for w in work {
        if w.is_substitution() {
            into_single_substitution(w, stmts);
        } else {
            stmts.push(w);
        }
    }
}

fn map_constraint_equalities(stmts: &mut Vec<Statement>) {
    let work = std::mem::take(stmts);
    for w in work {
        if w.is_constraint_equality() {
            into_single_constraint_equality(w, stmts);
        } else {
            stmts.push(w);
        }
    }
}

fn map_returns(stmts: &mut Vec<Statement>, mut fresh_id: usize) -> usize {
    use Statement::Return;
    let work = std::mem::take(stmts);
    for w in work {
        let should_split = match &w {
            Return { value, .. } => value.is_array() || value.is_switch() || value.is_call(),
            _ => false,
        };
        if should_split {
            let split = split_return(w, fresh_id);
            stmts.push(split.declaration);
            into_single_substitution(split.substitution, stmts);
            stmts.push(split.final_return);
            fresh_id += 1;
        } else {
            stmts.push(w);
        }
    }
    fresh_id
}

struct ReturnSplit {
    declaration: Statement,
    substitution: Statement,
    final_return: Statement,
}
fn split_return(stmt: Statement, id: usize) -> ReturnSplit {
    use num_bigint_dig::BigInt;
    use Expression::{Number, Variable};
    use Statement::{Declaration, Return, Substitution};
    if let Return { value, meta } = stmt {
        let lengths = value.get_meta().get_memory_knowledge().get_concrete_dimensions().to_vec();
        let expr_lengths: Vec<Expression> =
            lengths.iter().map(|v| Number(meta.clone(), BigInt::from(*v))).collect();
        let mut declaration_meta = meta.clone();
        let mut substitution_meta = meta.clone();
        let mut variable_meta = meta.clone();
        let return_meta = meta.clone();
        declaration_meta.get_mut_memory_knowledge().set_concrete_dimensions(lengths.clone());
        declaration_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
        substitution_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
        variable_meta.get_mut_memory_knowledge().set_concrete_dimensions(lengths);
        variable_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
        let declaration = Declaration {
            meta: declaration_meta,
            xtype: VariableType::Var,
            name: id.to_string(),
            dimensions: expr_lengths,
            is_constant: false,
        };
        let substitution = Substitution {
            meta: substitution_meta,
            var: id.to_string(),
            access: vec![],
            op: AssignOp::AssignVar,
            rhe: value,
        };
        let returned_variable =
            Variable { meta: variable_meta, name: id.to_string(), access: vec![] };
        let final_return = Return { meta: return_meta, value: returned_variable };
        ReturnSplit { declaration, substitution, final_return }
    } else {
        unreachable!()
    }
}

fn into_single_substitution(stmt: Statement, stmts: &mut Vec<Statement>) {
    use Statement::Substitution;
    match &stmt {
        Substitution { rhe, .. } if rhe.is_switch() => rhe_switch_case(stmt, stmts),
        Substitution { rhe, .. } if rhe.is_array() => rhe_array_case(stmt, stmts),
        _ => stmts.push(stmt),
    }
}

fn rhe_switch_case(stmt: Statement, stmts: &mut Vec<Statement>) {
    use Expression::InlineSwitchOp;
    use Statement::{Block, IfThenElse, Substitution};
    if let Substitution { var, access, op, rhe, meta } = stmt {
        if let InlineSwitchOp { cond, if_true, if_false, .. } = rhe {
            let mut if_assigns = vec![];
            let sub_if = Substitution {
                meta: meta.clone(),
                var: var.clone(),
                access: access.clone(),
                op: op.clone(),
                rhe: *if_true,
            };
            if_assigns.push(sub_if);
            let mut else_assigns = vec![];
            let sub_else = Substitution { op, var, access, meta: meta.clone(), rhe: *if_false };
            else_assigns.push(sub_else);
            let if_body = Block { stmts: if_assigns, meta: meta.clone() };
            let else_body = Block { stmts: else_assigns, meta: meta.clone() };
            let stmt = IfThenElse {
                meta,
                cond: *cond,
                if_case: Box::new(if_body),
                else_case: Option::Some(Box::new(else_body)),
            };
            stmts.push(stmt);
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}

fn cast_dimension(ae_index: &Expression) -> Option<usize> {
    use Expression::Number;

    if let Number(_, value) = ae_index {
        value.to_usize()
    } else {
        Option::None
    }
}

fn rhe_array_case(stmt: Statement, stmts: &mut Vec<Statement>) {
    use num_bigint_dig::BigInt;
    use Expression::{ArrayInLine, Number, UniformArray};
    use Statement::Substitution;
    if let Substitution { var, access, op, rhe, meta } = stmt {
        if let ArrayInLine { values, .. } = rhe {
            let mut index = 0;
            for v in values {
                let mut index_meta = meta.clone();
                index_meta.get_mut_memory_knowledge().set_concrete_dimensions(vec![]);
                let expr_index = Number(index_meta, BigInt::from(index));
                let as_access = Access::ArrayAccess(expr_index);
                let mut accessed_with = access.clone();
                accessed_with.push(as_access);
                let sub = Substitution {
                    op,
                    var: var.clone(),
                    access: accessed_with,
                    meta: meta.clone(),
                    rhe: v,
                };
                stmts.push(sub);
                index += 1;
            }
        } else if let UniformArray { value, dimension, .. } = rhe {
            let usable_dimension = if let Option::Some(dimension) = cast_dimension(&dimension) {
                dimension
            } else {
                unreachable!()
            };
            let mut index: usize = 0;
            while index < usable_dimension {
                let mut index_meta = meta.clone();
                index_meta.get_mut_memory_knowledge().set_concrete_dimensions(vec![]);
                let expr_index = Number(index_meta, BigInt::from(index));
                let as_access = Access::ArrayAccess(expr_index);
                let mut accessed_with = access.clone();
                accessed_with.push(as_access);
                let sub = Substitution {
                    op,
                    var: var.clone(),
                    access: accessed_with,
                    meta: meta.clone(),
                    rhe: *value.clone(),
                };
                stmts.push(sub);
                index += 1;
            }
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}

fn into_single_constraint_equality(stmt: Statement, stmts: &mut Vec<Statement>) {
    use Statement::ConstraintEquality;
    match &stmt {
        ConstraintEquality { rhe, lhe, meta, .. } =>{
            if lhe.is_array() {
                lhe_array_ce(meta, lhe, rhe, stmts)
            }
            else if rhe.is_array(){
                lhe_array_ce(meta, rhe, lhe, stmts)
            }
            else{
                stmts.push(stmt);
            }
        }
        _ => stmts.push(stmt),
    }
}

fn lhe_array_ce(meta: &Meta, expr_array: &Expression, other_expr: &Expression, stmts: &mut Vec<Statement>) {
    use num_bigint_dig::BigInt;
    use Expression::{ArrayInLine, Number, UniformArray, Variable};
    use Statement::ConstraintEquality;
    if let ArrayInLine { values: values_l, .. } = expr_array {
        if let ArrayInLine { values: values_r, .. } = other_expr {
            for i in 0..values_l.len() {
                let ce = ConstraintEquality {
                    lhe: values_l[i].clone(),
                    rhe: values_r[i].clone(),
                    meta: meta.clone(),
                };
                stmts.push(ce);
            }
        }
        else if let UniformArray {value, ..} = other_expr {
            for i in 0..values_l.len() {
                let ce = ConstraintEquality {
                    lhe: values_l[i].clone(),
                    rhe: *value.clone(),
                    meta: meta.clone(),
                };
                stmts.push(ce);
            }
        }
        else if let Variable { name, access, ..} = other_expr {
            for i in 0..values_l.len() {
                let mut index_meta = meta.clone();
                index_meta.get_mut_memory_knowledge().set_concrete_dimensions(vec![]);
                let expr_index = Number(index_meta, BigInt::from(i));
                let as_access = Access::ArrayAccess(expr_index);
                let mut accessed_with = access.clone();
                accessed_with.push(as_access);
                let ce = ConstraintEquality {
                    lhe: values_l[i].clone(),
                    rhe: Variable {name: name.clone(), access: accessed_with, meta: meta.clone()},
                    meta: meta.clone(),
                };
                stmts.push(ce);
            }
        }
        
    } else {
        unreachable!()
    }
}