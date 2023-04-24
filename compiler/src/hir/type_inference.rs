use super::analysis_utilities::*;
use super::very_concrete_program::*;
use program_structure::ast::*;
use std::collections::HashSet;
use num_traits::{ToPrimitive};

struct SearchInfo {
    environment: E,
    open_calls: HashSet<String>,
}

pub fn infer_function_result(id: &str, params: Vec<Param>, state: &State) -> VCT {
    let body = &state.generic_functions.get(id).unwrap().body;
    let mut context = SearchInfo { environment: E::new(), open_calls: HashSet::new() };
    context.open_calls.insert(id.to_string());
    for p in params {
        context.environment.add_variable(&p.name, p.length);
    }
    infer_type_stmt(body, state, &mut context).unwrap()
}

fn infer_type_stmt(stmt: &Statement, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    if stmt.is_return() {
        infer_type_return(stmt, state, context)
    } else if stmt.is_block() {
        infer_type_block(stmt, state, context)
    } else if stmt.is_initialization_block() {
        infer_type_init_block(stmt, state, context)
    } else if stmt.is_if_then_else() {
        infer_type_conditional(stmt, state, context)
    } else if stmt.is_while() {
        infer_type_while(stmt, state, context)
    } else if stmt.is_declaration() {
        infer_type_declaration(stmt, state, context)
    } else if stmt.is_substitution() {
        Option::None
    } else if stmt.is_constraint_equality() {
        Option::None
    } else if stmt.is_log_call() {
        Option::None
    } else if stmt.is_assert() {
        Option::None
    } else{
        unreachable!()
    }
}

fn infer_type_return(stmt: &Statement, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Statement::Return;
    if let Return { value, .. } = stmt {
        infer_type_expresion(value, state, context)
    } else {
        unreachable!();
    }
}

fn infer_type_declaration(
    stmt: &Statement,
    _state: &State,
    context: &mut SearchInfo,
) -> Option<VCT> {
    use Statement::Declaration;
    if let Declaration { meta, name, .. } = stmt {
        let has_type = meta.get_memory_knowledge().get_concrete_dimensions().to_vec();
        context.environment.add_variable(name, has_type);
        Option::None
    } else {
        unreachable!();
    }
}

fn infer_type_block(stmt: &Statement, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Statement::Block;
    if let Block { stmts, .. } = stmt {
        let mut returns = Option::None;
        let mut index = 0;
        context.environment.add_variable_block();
        while index < stmts.len() && returns.is_none() {
            returns = infer_type_stmt(&stmts[index], state, context);
            index += 1;
        }
        context.environment.remove_variable_block();
        returns
    } else {
        unreachable!();
    }
}

fn infer_type_init_block(stmt: &Statement, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Statement::InitializationBlock;
    if let InitializationBlock { initializations, .. } = stmt {
        for s in initializations {
            if s.is_declaration() {
                infer_type_declaration(s, state, context);
            }
        }
        Option::None
    } else {
        unreachable!();
    }
}

fn infer_type_conditional(
    stmt: &Statement,
    state: &State,
    context: &mut SearchInfo,
) -> Option<VCT> {
    use Statement::IfThenElse;
    if let IfThenElse { if_case, else_case, .. } = stmt {
        let mut returns = infer_type_stmt(if_case, state, context);
        if let Option::Some(s) = else_case {
            if returns.is_none() {
                returns = infer_type_stmt(s, state, context);
            }
        }
        returns
    } else {
        unreachable!();
    }
}

fn infer_type_while(stmt: &Statement, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Statement::While;
    if let While { stmt, .. } = stmt {
        infer_type_stmt(stmt, state, context)
    } else {
        unreachable!();
    }
}

fn infer_type_expresion(expr: &Expression, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    if expr.is_call() {
        infer_type_call(expr, state, context)
    } else if expr.is_variable() {
        infer_type_variable(expr, state, context)
    } else if expr.is_array() {
        infer_type_array(expr, state, context)
    } else if expr.is_switch() {
        infer_type_switch(expr, state, context)
    } else if expr.is_infix() {
        Option::Some(VCT::with_capacity(0))
    } else if expr.is_prefix() {
        Option::Some(VCT::with_capacity(0))
    } else if expr.is_parallel(){
        infer_type_parallel(expr, state, context)
    } else if expr.is_number() {
        Option::Some(VCT::with_capacity(0))
    } else {
        unreachable!()
    }
}

fn infer_type_switch(expr: &Expression, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Expression::InlineSwitchOp;
    if let InlineSwitchOp { if_true, if_false, .. } = expr {
        let casts_to = infer_type_expresion(if_true, state, context);
        if casts_to.is_none() {
            infer_type_expresion(if_false, state, context)
        } else {
            casts_to
        }
    } else {
        unreachable!()
    }
}

fn infer_type_parallel(expr: &Expression, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Expression::ParallelOp;
    if let ParallelOp { rhe, .. } = expr {
        infer_type_expresion(rhe, state, context)
    } else {
        unreachable!()
    }
}

fn infer_type_variable(expr: &Expression, _state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Expression::Variable;
    if let Variable { name, access, .. } = expr {
        let with_type = context.environment.get_variable(name).unwrap();
        Option::Some(with_type[access.len()..].to_vec())
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

fn infer_type_array(expr: &Expression, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Expression::{ArrayInLine, UniformArray};
    if let ArrayInLine { values, .. } = expr {
        let mut lengths = vec![values.len()];
        let with_type = infer_type_expresion(&values[0], state, context);
        with_type.map(|mut l| {
            lengths.append(&mut l);
            lengths
        })
    } else if let UniformArray { value, dimension, .. } = expr {
        let usable_dimension = if let Option::Some(dimension) = cast_dimension(&dimension) {
            dimension
        } else {
            unreachable!()
        };
        let mut lengths = vec![usable_dimension];
        let with_type = infer_type_expresion(&value, state, context);
        with_type.map(|mut l| {
            lengths.append(&mut l);
            lengths
        })
    } else {
        unreachable!()
    }
}

fn infer_type_call(expr: &Expression, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Expression::Call;
    if let Call { id, args, .. } = expr {
        if context.open_calls.contains(id) {
            Option::None
        } else {
            context.open_calls.insert(id.clone());
            context.environment.add_variable_block();
            let mut index = 0;
            let body = &state.generic_functions.get(id).unwrap().body;
            let names = &state.generic_functions.get(id).unwrap().params_names;
            let arg_types = infer_args(args, state, context);
            if arg_types.is_none() {
                return Option::None;
            }
            let arg_types = arg_types.unwrap();
            for arg_type in arg_types {
                context.environment.add_variable(&names[index], arg_type);
                index += 1;
            }
            let inferred = infer_type_stmt(body, state, context);
            context.environment.remove_variable_block();
            context.open_calls.remove(id);
            inferred
        }
    } else {
        unreachable!()
    }
}

fn infer_args(args: &[Expression], state: &State, context: &mut SearchInfo) -> Option<Vec<VCT>> {
    let mut arg_types = vec![];
    let mut index = 0;
    loop {
        if index == args.len() {
            break Option::Some(arg_types);
        }
        let arg_type = infer_type_expresion(&args[index], state, context);
        if let Option::Some(t) = arg_type {
            arg_types.push(t);
        } else {
            break Option::None;
        }
        index += 1;
    }
}
