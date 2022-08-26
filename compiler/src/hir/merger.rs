use super::analysis_utilities::*;
use super::component_preprocess;
use super::sugar_cleaner;
use super::very_concrete_program::*;
use program_structure::ast::*;
use program_structure::program_archive::ProgramArchive;
use num_traits::{ToPrimitive};


pub fn run_preprocessing(vcp: &mut VCP, program_archive: ProgramArchive) {
    let mut state = build_function_knowledge(program_archive);
    produce_vcf(vcp, &mut state);
    link_circuit(vcp, &mut state);
    vcp.quick_knowledge = state.quick_knowledge;
    vcp.functions = state.vcf_collector;
    component_preprocess::rm_component_ci(vcp);
    sugar_cleaner::clean_sugar(vcp);
}

fn produce_vcf(vcp: &VCP, state: &mut State) {
    for n in &vcp.templates {
        let code = &n.code;
        let constants = &n.header;
        let params = vec![];
        state.external_signals = build_component_info(&n.triggers);
        let mut env = build_environment(constants, &params);
        produce_vcf_stmt(code, state, &mut env);
    }
    let mut index = 0;
    while index < state.vcf_collector.len() {
        state.external_signals = build_component_info(&vec![]);
        let mut env = build_environment(&vec![], &state.vcf_collector[index].params_types);
        let body = state.vcf_collector[index].body.clone();
        produce_vcf_stmt(&body, state, &mut env);
        index += 1;
    }
}

fn link_circuit(vcp: &mut VCP, state: &mut State) {
    for node in &mut vcp.templates {
        let mut env = build_environment(&node.header, &vec![]);
        state.external_signals = build_component_info(&node.triggers);
        link_stmt(&mut node.code, state, &mut env);
    }
    let mut linked_vcf_collector = state.vcf_collector.clone();
    for vcf in &mut linked_vcf_collector {
        let mut env = build_environment(&Vec::new(), &vcf.params_types);
        link_stmt(&mut vcf.body, state, &mut env);
    }
    state.vcf_collector = linked_vcf_collector;
}

// Function knowledge handling
fn add_instance(name: &str, args: Vec<Param>, state: &mut State) {
    use super::type_inference::infer_function_result;
    let already_exits = look_for_existing_instance(name, &args, state);
    if already_exits.is_none() {
        let inferred = infer_function_result(name, args.clone(), state);
        let id = state.vcf_collector.len();
        let body = state.generic_functions.get(name).unwrap().body.clone();
        let new_vcf = VCF {
            name: name.to_string(),
            header: format!("{}_{}", name, state.vcf_collector.len()),
            params_types: args.to_vec(),
            return_type: inferred,
            body,
        };
        state.quick_knowledge.insert(new_vcf.header.clone(), new_vcf.return_type.clone());
        state.vcf_collector.push(new_vcf);
        state.generic_functions.get_mut(name).unwrap().concrete_instances.push(id);
    }
}

fn look_for_existing_instance(
    name: &str,
    args: &Vec<Param>,
    state: &State,
) -> Option<(usize, VCT)> {
    let vcf = state.generic_functions.get(name).unwrap();
    for i in &vcf.concrete_instances {
        let return_type = &state.vcf_collector[*i].return_type;
        let params_types = &state.vcf_collector[*i].params_types;
        if params_types.eq(args) {
            return Option::Some((*i, return_type.clone()));
        }
    }
    Option::None
}

// Algorithm for producing the code's vcf
fn produce_vcf_stmt(stmt: &Statement, state: &mut State, environment: &mut E) {
    if stmt.is_return() {
        produce_vcf_return(stmt, state, environment);
    } else if stmt.is_assert() {
        produce_vcf_assert(stmt, state, environment);
    } else if stmt.is_log_call() {
        produce_vcf_log_call(stmt, state, environment);
    } else if stmt.is_constraint_equality() {
        produce_vcf_constraint_equality(stmt, state, environment);
    } else if stmt.is_substitution() {
        produce_vcf_substitution(stmt, state, environment);
    } else if stmt.is_declaration() {
        produce_vcf_declaration(stmt, state, environment);
    } else if stmt.is_block() {
        produce_vcf_block(stmt, state, environment);
    } else if stmt.is_initialization_block() {
        produce_vcf_init_block(stmt, state, environment);
    } else if stmt.is_while() {
        produce_vcf_while(stmt, state, environment);
    } else if stmt.is_if_then_else() {
        produce_vcf_conditional(stmt, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_expr(expr: &Expression, state: &mut State, environment: &E) {
    if expr.is_infix() {
        produce_vcf_infix(expr, state, environment);
    } else if expr.is_prefix() {
        produce_vcf_prefix(expr, state, environment);
    } else if expr.is_switch() {
        produce_vcf_switch(expr, state, environment);
    } else if expr.is_variable() {
        produce_vcf_variable(expr, state, environment);
    } else if expr.is_number() {
        produce_vcf_number(expr, state, environment);
    } else if expr.is_call() {
        produce_vcf_call(expr, state, environment);
    } else if expr.is_array() {
        produce_vcf_array(expr, state, environment);
    } else if expr.is_parallel(){
        produce_vcf_parallel(expr, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_return(stmt: &Statement, state: &mut State, environment: &E) {
    use Statement::Return;
    if let Return { value, .. } = stmt {
        produce_vcf_expr(value, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_assert(stmt: &Statement, state: &mut State, environment: &E) {
    use Statement::Assert;
    if let Assert { arg, .. } = stmt {
        produce_vcf_expr(arg, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_log_call(stmt: &Statement, state: &mut State, environment: &E) {
    use Statement::LogCall;
    if let LogCall { args, .. } = stmt {
        for arglog in args {
            if let LogArgument::LogExp(arg) = arglog {
                produce_vcf_expr(arg, state, environment);
            }
            else {}// unimplemented!(); }
        }
    } else {
        unreachable!();
    }
}

fn produce_vcf_constraint_equality(stmt: &Statement, state: &mut State, environment: &E) {
    use Statement::ConstraintEquality;
    if let ConstraintEquality { lhe, rhe, .. } = stmt {
        produce_vcf_expr(lhe, state, environment);
        produce_vcf_expr(rhe, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_substitution(stmt: &Statement, state: &mut State, environment: &E) {
    use Statement::Substitution;
    if let Substitution { access, rhe, .. } = stmt {
        produce_vcf_expr(rhe, state, environment);
        for a in access {
            if let Access::ArrayAccess(index) = a {
                produce_vcf_expr(index, state, environment);
            }
        }
    } else {
        unreachable!();
    }
}

fn produce_vcf_declaration(stmt: &Statement, state: &mut State, environment: &mut E) {
    use Statement::Declaration;
    if let Declaration { name, dimensions, meta, .. } = stmt {
        for d in dimensions {
            produce_vcf_expr(d, state, environment);
        }
        let with_type = meta.get_memory_knowledge().get_concrete_dimensions().to_vec();
        environment.add_variable(name, with_type);
    } else {
        unreachable!();
    }
}

fn produce_vcf_block(stmt: &Statement, state: &mut State, environment: &mut E) {
    use Statement::Block;
    if let Block { stmts, .. } = stmt {
        environment.add_variable_block();
        for s in stmts {
            produce_vcf_stmt(s, state, environment);
        }
        environment.remove_variable_block();
    } else {
        unreachable!();
    }
}

fn produce_vcf_init_block(stmt: &Statement, state: &mut State, environment: &mut E) {
    use Statement::InitializationBlock;
    if let InitializationBlock { initializations, .. } = stmt {
        for s in initializations {
            produce_vcf_stmt(s, state, environment);
        }
    } else {
        unreachable!();
    }
}

fn produce_vcf_while(stmt: &Statement, state: &mut State, environment: &mut E) {
    use Statement::While;
    if let While { cond, stmt, .. } = stmt {
        produce_vcf_expr(cond, state, environment);
        produce_vcf_stmt(stmt, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_conditional(stmt: &Statement, state: &mut State, environment: &mut E) {
    use Statement::IfThenElse;
    if let IfThenElse { cond, if_case, else_case, .. } = stmt {
        produce_vcf_expr(cond, state, environment);
        produce_vcf_stmt(if_case, state, environment);
        if let Option::Some(s) = else_case {
            produce_vcf_stmt(s, state, environment);
        }
    }
}

fn produce_vcf_array(expr: &Expression, state: &mut State, environment: &E) {
    use Expression::{ArrayInLine, UniformArray};
    if let ArrayInLine { values, .. } = expr {
        for v in values {
            produce_vcf_expr(v, state, environment);
        }
    } else if let UniformArray { value, dimension, .. } = expr {
        produce_vcf_expr(value, state, environment);
        produce_vcf_expr(dimension, state, environment);
    } else {
        unreachable!();
    }
}
fn produce_vcf_call(expr: &Expression, state: &mut State, environment: &E) {
    use Expression::Call;
    if let Call { id, args, .. } = expr {
        for arg in args {
            produce_vcf_expr(arg, state, environment);
        }
        if state.generic_functions.contains_key(id) {
            let params = map_to_params(id, args, state, environment);
            add_instance(id, params, state);
        }
    } else {
        unreachable!();
    }
}

fn produce_vcf_variable(expr: &Expression, state: &mut State, environment: &E) {
    use Access::ArrayAccess;
    use Expression::Variable;
    if let Variable { access, .. } = expr {
        for a in access {
            if let ArrayAccess(index) = a {
                produce_vcf_expr(index, state, environment);
            }
        }
    } else {
        unreachable!();
    }
}

fn produce_vcf_switch(expr: &Expression, state: &mut State, environment: &E) {
    use Expression::InlineSwitchOp;
    if let InlineSwitchOp { if_true, if_false, .. } = expr {
        produce_vcf_expr(if_true, state, environment);
        produce_vcf_expr(if_false, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_infix(expr: &Expression, state: &mut State, environment: &E) {
    use Expression::InfixOp;
    if let InfixOp { lhe, rhe, .. } = expr {
        produce_vcf_expr(lhe, state, environment);
        produce_vcf_expr(rhe, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_prefix(expr: &Expression, state: &mut State, environment: &E) {
    use Expression::PrefixOp;
    if let PrefixOp { rhe, .. } = expr {
        produce_vcf_expr(rhe, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_parallel(expr: &Expression, state: &mut State, environment: &E) {
    use Expression::ParallelOp;
    if let ParallelOp { rhe, .. } = expr {
        produce_vcf_expr(rhe, state, environment);
    } else {
        unreachable!();
    }
}

fn produce_vcf_number(_expr: &Expression, _state: &State, _environment: &E) {}

/*
    Linking algorithm:
    The linking algorithm not only changes
    functions call to vcf calls. Also, every expression
    will be linked with its vct (very concrete type).
*/
fn link_stmt(stmt: &mut Statement, state: &State, env: &mut E) {
    if stmt.is_block() {
        link_block(stmt, state, env);
    } else if stmt.is_initialization_block() {
        link_initialization_block(stmt, state, env);
    } else if stmt.is_if_then_else() {
        link_conditional(stmt, state, env);
    } else if stmt.is_while() {
        link_while(stmt, state, env);
    } else if stmt.is_log_call() {
        link_log_call(stmt, state, env);
    } else if stmt.is_assert() {
        link_assert(stmt, state, env);
    } else if stmt.is_return() {
        link_return(stmt, state, env);
    } else if stmt.is_constraint_equality() {
        link_constraint_eq(stmt, state, env);
    } else if stmt.is_declaration() {
        link_declaration(stmt, state, env);
    } else if stmt.is_substitution() {
        link_substitution(stmt, state, env);
    } else {
        unreachable!();
    }
}

fn link_block(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::Block;
    if let Block { stmts, .. } = stmt {
        env.add_variable_block();
        for s in stmts {
            link_stmt(s, state, env);
        }
        env.remove_variable_block();
    } else {
        unreachable!();
    }
}

fn link_initialization_block(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::InitializationBlock;
    if let InitializationBlock { initializations, .. } = stmt {
        for i in initializations {
            link_stmt(i, state, env);
        }
    } else {
        unreachable!();
    }
}

fn link_conditional(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::IfThenElse;
    if let IfThenElse { cond, if_case, else_case, .. } = stmt {
        link_expression(cond, state, env);
        link_stmt(if_case, state, env);
        if let Option::Some(s) = else_case {
            link_stmt(s, state, env);
        }
    } else {
        unreachable!();
    }
}

fn link_while(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::While;
    if let While { cond, stmt, .. } = stmt {
        link_expression(cond, state, env);
        link_stmt(stmt, state, env);
    } else {
        unreachable!();
    }
}

fn link_log_call(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::LogCall;
    if let LogCall { args, .. } = stmt {
        for arglog in args {
            if let LogArgument::LogExp(arg) = arglog{
                link_expression(arg, state, env);
            }
        }   
    } else {
        unreachable!();
    }
}

fn link_assert(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::Assert;
    if let Assert { arg, .. } = stmt {
        link_expression(arg, state, env);
    } else {
        unreachable!();
    }
}

fn link_return(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::Return;
    if let Return { value, .. } = stmt {
        link_expression(value, state, env);
    } else {
        unreachable!();
    }
}

fn link_constraint_eq(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::ConstraintEquality;
    if let ConstraintEquality { lhe, rhe, .. } = stmt {
        link_expression(lhe, state, env);
        link_expression(rhe, state, env);
    } else {
        unreachable!();
    }
}

fn link_declaration(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::Declaration;
    if let Declaration { name, dimensions, meta, .. } = stmt {
        for d in dimensions {
            link_expression(d, state, env);
        }
        let has_type = meta.get_memory_knowledge().get_concrete_dimensions().to_vec();
        env.add_variable(name, has_type);
    } else {
        unreachable!();
    }
}

fn link_substitution(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::Substitution;
    if let Substitution { access, rhe, .. } = stmt {
        link_expression(rhe, state, env);
        for acc in access {
            if let Access::ArrayAccess(e) = acc {
                link_expression(e, state, env);
            }
        }
    }
}

fn link_expression(expr: &mut Expression, state: &State, env: &E) {
    if expr.is_call() {
        link_call(expr, state, env);
    } else if expr.is_array() {
        link_array(expr, state, env);
    } else if expr.is_variable() {
        link_variable(expr, state, env);
    } else if expr.is_switch() {
        link_switch(expr, state, env);
    } else if expr.is_number() {
        link_number(expr, state, env);
    } else if expr.is_infix() {
        link_infix(expr, state, env);
    } else if expr.is_prefix() {
        link_prefix(expr, state, env);
    } else if expr.is_parallel(){
        link_parallel(expr, state, env);
    } else {
        unreachable!();
    }
    let has_type = cast_type_expression(expr, state, env);
    expr.get_mut_meta().get_mut_memory_knowledge().set_concrete_dimensions(has_type);
}

fn link_call(expr: &mut Expression, state: &State, env: &E) {
    use Expression::Call;
    if let Call { id, args, .. } = expr {
        for arg in args.iter_mut() {
            link_expression(arg, state, env);
        }
        if state.generic_functions.contains_key(id) {
            let params = map_to_params(id, args, state, env);
            let (index, _) = look_for_existing_instance(id, &params, state).unwrap();
            *id = state.vcf_collector[index].header.clone();
        }
    } else {
        unreachable!();
    }
}

fn link_array(expr: &mut Expression, state: &State, env: &E) {
    use Expression::{ArrayInLine, UniformArray};
    if let ArrayInLine { values, .. } = expr {
        for v in values {
            link_expression(v, state, env)
        }
    } else if let UniformArray { value, dimension, .. } = expr {
        link_expression(value, state, env);
        link_expression(dimension, state, env);
    } else {
        unreachable!();
    }
}

fn link_variable(expr: &mut Expression, state: &State, env: &E) {
    use Expression::Variable;
    if let Variable { access, .. } = expr {
        for acc in access {
            if let Access::ArrayAccess(e) = acc {
                link_expression(e, state, env);
            }
        }
    } else {
        unreachable!();
    }
}

fn link_switch(expr: &mut Expression, state: &State, env: &E) {
    use Expression::InlineSwitchOp;
    if let InlineSwitchOp { if_true, if_false, .. } = expr {
        link_expression(if_true, state, env);
        link_expression(if_false, state, env);
    } else {
        unreachable!();
    }
}

fn link_number(_expr: &mut Expression, _state: &State, _env: &E) {}

fn link_infix(expr: &mut Expression, state: &State, env: &E) {
    use Expression::InfixOp;
    if let InfixOp { lhe, rhe, .. } = expr {
        link_expression(lhe, state, env);
        link_expression(rhe, state, env);
    } else {
        unreachable!();
    }
}

fn link_prefix(expr: &mut Expression, state: &State, env: &E) {
    use Expression::PrefixOp;
    if let PrefixOp { rhe, .. } = expr {
        link_expression(rhe, state, env);
    } else {
        unreachable!();
    }
}

fn link_parallel(expr: &mut Expression, state: &State, env: &E) {
    use Expression::ParallelOp;
    if let ParallelOp { rhe, .. } = expr {
        link_expression(rhe, state, env);
    } else {
        unreachable!();
    }
}

// When the vcf of a branch in the AST had been produced, this algorithm
// can return the type of a expression in that branch
fn cast_type_expression(expr: &Expression, state: &State, environment: &E) -> VCT {
    if expr.is_variable() {
        cast_type_variable(expr, state, environment)
    } else if expr.is_array() {
        cast_type_array(expr, state, environment)
    } else if expr.is_call() {
        cast_type_call(expr, state, environment)
    } else if expr.is_switch() {
        cast_type_switch(expr, state, environment)
    } else {
        VCT::with_capacity(0)
    }
}

fn cast_type_variable(expr: &Expression, state: &State, environment: &E) -> VCT {
    use Expression::Variable;
    if let Variable { name, access, .. } = expr {
        let mut xtype = environment.get_variable(name).unwrap().clone();
        xtype.reverse();
        for acc in access {
            match acc {
                Access::ArrayAccess(_) => {
                    xtype.pop();
                }
                Access::ComponentAccess(signal) => {
                    xtype = state.external_signals.get(name).unwrap().get(signal).unwrap().clone();
                    xtype.reverse();
                }
            }
        }
        xtype.reverse();
        xtype
    } else {
        unreachable!();
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


fn cast_type_array(expr: &Expression, state: &State, environment: &E) -> VCT {
    use Expression::{ArrayInLine, UniformArray};
    if let ArrayInLine { values, .. } = expr {
        let mut result = vec![values.len()];
        let mut inner_type = cast_type_expression(&values[0], state, environment);
        result.append(&mut inner_type);
        result
    } else if let UniformArray { value, dimension, .. } = expr {
        let usable_dimension = if let Option::Some(dimension) = cast_dimension(&dimension) {
            dimension
        } else {
            unreachable!()
        };
        let mut lengths = vec![usable_dimension];
        let mut inner_type = cast_type_expression(value, state, environment);
        lengths.append(&mut inner_type);
        lengths
    } else {
        unreachable!();
    }
}

fn cast_type_call(expr: &Expression, state: &State, environment: &E) -> VCT {
    use Expression::Call;
    if let Call { id, args, .. } = expr {
        if let Option::Some(returns) = state.quick_knowledge.get(id) {
            returns.clone()
        } else if !state.generic_functions.contains_key(id) {
            vec![]
        } else {
            let params = map_to_params(id, args, state, environment);
            look_for_existing_instance(id, &params, state).unwrap().1
        }
    } else {
        unreachable!()
    }
}

fn cast_type_switch(expr: &Expression, state: &State, environment: &E) -> VCT {
    use Expression::InlineSwitchOp;
    if let InlineSwitchOp { if_true, .. } = expr {
        cast_type_expression(if_true, state, environment)
    } else {
        unreachable!();
    }
}

fn map_to_params(function_id: &str, args: &[Expression], state: &State, env: &E) -> Vec<Param> {
    let mut params = vec![];
    let names = &state.generic_functions.get(function_id).unwrap().params_names;
    let mut index = 0;
    while index < args.len() {
        let param = Param {
            name: names[index].clone(),
            length: cast_type_expression(&args[index], state, env),
        };
        params.push(param);
        index += 1;
    }
    params
}
