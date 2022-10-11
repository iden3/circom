use num_bigint::BigInt;
use program_structure::ast::*;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::expression_builders::*;
use program_structure::utils::environment::VarEnvironment;
use program_structure::{function_data::FunctionData, template_data::TemplateData};
use std::collections::HashSet;

type Constants = VarEnvironment<bool>;
type ExpressionHolder = VarEnvironment<Expression>;

pub fn handle_function_constants(function: &mut FunctionData) -> ReportCollection {
    let mut environment = Constants::new();
    let mut expression_holder = ExpressionHolder::new();
    for p in function.get_name_of_params() {
        environment.add_variable(p, false);
    }
    statement_constant_inference(function.get_mut_body(), &mut environment);
    let reports = statement_invariant_check(function.get_body(), &mut environment);
    expand_statement(function.get_mut_body(), &mut expression_holder);
    reports
}
pub fn _handle_template_constants(template: &mut TemplateData) -> ReportCollection {
    let mut environment = Constants::new();
    let mut expression_holder = ExpressionHolder::new();
    for p in template.get_name_of_params() {
        environment.add_variable(p, true);
        let meta = template.get_body().get_meta().clone();
        let name = p.clone();
        let access = vec![];
        let expression = build_variable(meta, name, access);
        expression_holder.add_variable(p, expression);
    }
    statement_constant_inference(template.get_mut_body(), &mut environment);
    let reports = statement_invariant_check(template.get_body(), &mut environment);
    expand_statement(template.get_mut_body(), &mut expression_holder);
    reports
}

// Set of functions used to infer the constant tag in variable declarations
fn statement_constant_inference(stmt: &mut Statement, environment: &mut Constants) {
    use Statement::*;
    match stmt {
        IfThenElse { if_case, else_case, .. } => {
            if_then_else_constant_inference(if_case, else_case, environment)
        }
        Substitution { var, .. } => substitution_constant_inference(var, environment),
        InitializationBlock { initializations, .. } => {
            initialization_block_constant_inference(initializations, environment)
        }
        While { stmt, .. } => while_stmt_constant_inference(stmt, environment),
        Block { stmts, .. } => block_constant_inference(stmts, environment),
        _ => {}
    }
}

fn declaration_constant_inference(name: &str, is_constant: bool, environment: &mut Constants) {
    environment.add_variable(name, is_constant);
}

fn substitution_constant_inference(name: &str, environment: &mut Constants) {
    *environment.get_mut_variable_or_break(name, file!(), line!()) = false;
}

fn initialization_block_constant_inference(
    initializations: &[Statement],
    environment: &mut Constants,
) {
    use Statement::{Declaration, Substitution};
    let mut initialized_signals = HashSet::new();
    for s in initializations {
        if let Substitution { var, .. } = s {
            initialized_signals.insert(var.clone());
        }
    }
    for s in initializations {
        if let Declaration { name, xtype, dimensions, .. } = s {
            let constant_tag = dimensions.is_empty()
                && initialized_signals.contains(name)
                && (*xtype == VariableType::Var);
            declaration_constant_inference(name, constant_tag, environment);
        }
    }
}

fn while_stmt_constant_inference(stmt: &mut Statement, environment: &mut Constants) {
    statement_constant_inference(stmt, environment)
}

fn if_then_else_constant_inference(
    if_case: &mut Statement,
    else_case: &mut Option<Box<Statement>>,
    environment: &mut Constants,
) {
    statement_constant_inference(if_case, environment);
    if let Option::Some(stmt) = else_case {
        statement_constant_inference(stmt.as_mut(), environment);
    }
}

fn apply_inference(stmts: &mut [Statement], environment: &mut Constants) {
    use Statement::{Declaration, Substitution};
    for s in stmts.iter_mut() {
        if let Substitution { var, rhe, .. } = s {
            let was_constant = *environment.get_variable_or_break(var, file!(), line!());
            let remains_constant = has_constant_value(rhe, environment);
            *environment.get_mut_variable_or_break(var, file!(), line!()) =
                was_constant && remains_constant;
        }
    }
    for s in stmts.iter_mut() {
        if let Declaration { name, is_constant, .. } = s {
            let constant_indeed = *environment.get_variable_or_break(name, file!(), line!());
            *is_constant = constant_indeed;
        }
    }
}

fn block_constant_inference(stmts: &mut [Statement], environment: &mut Constants) {
    use Statement::InitializationBlock;
    environment.add_variable_block();
    for stmt in stmts.iter_mut() {
        statement_constant_inference(stmt, environment);
    }
    for stmt in stmts.iter_mut() {
        if let InitializationBlock { initializations, .. } = stmt {
            apply_inference(initializations, environment);
        }
    }
    environment.remove_variable_block();
}

// Set of functions whose purpose is to check the array length invariant
fn statement_invariant_check(stmt: &Statement, environment: &mut Constants) -> ReportCollection {
    use Statement::*;
    match stmt {
        InitializationBlock { initializations, .. } => {
            initialization_invariant_check(initializations, environment)
        }
        IfThenElse { if_case, else_case, .. } => {
            if_then_else_invariant_check(if_case, else_case, environment)
        }
        While { stmt, .. } => while_invariant_check(stmt, environment),
        Block { stmts, .. } => block_invariant_check(stmts, environment),
        MultSubstitution { .. } => unreachable!(),
        _ => ReportCollection::new(),
    }
}

fn declaration_invariant_check(
    dimensions: &[Expression],
    environment: &mut Constants,
) -> ReportCollection {
    let mut reports = ReportCollection::new();
    for d in dimensions {
        if !has_constant_value(d, environment) {
            broken_invariant_error(d.get_meta(), &mut reports);
        }
    }
    reports
}

fn initialization_invariant_check(
    initializations: &[Statement],
    environment: &mut Constants,
) -> ReportCollection {
    use Statement::Declaration;
    let mut reports = ReportCollection::new();
    for init in initializations {
        if let Declaration { dimensions, .. } = init {
            let mut r = declaration_invariant_check(dimensions, environment);
            reports.append(&mut r);
        }
    }
    for init in initializations {
        if let Declaration { name, is_constant, .. } = init {
            environment.add_variable(name, *is_constant);
        }
    }
    reports
}

fn if_then_else_invariant_check(
    if_case: &Statement,
    else_case: &Option<Box<Statement>>,
    environment: &mut Constants,
) -> ReportCollection {
    let mut reports = statement_invariant_check(if_case, environment);
    if let Option::Some(s) = else_case {
        let mut r = statement_invariant_check(s, environment);
        reports.append(&mut r);
    }
    reports
}

fn while_invariant_check(body: &Statement, environment: &mut Constants) -> ReportCollection {
    statement_invariant_check(body, environment)
}

fn block_invariant_check(stmts: &[Statement], environment: &mut Constants) -> ReportCollection {
    let mut reports = ReportCollection::new();
    environment.add_variable_block();
    for s in stmts {
        let mut r = statement_invariant_check(s, environment);
        reports.append(&mut r);
    }
    environment.remove_variable_block();
    reports
}

// Set of functions whose purpose is to determine if a expression is constant or not under some environment
fn has_constant_value(expr: &Expression, environment: &Constants) -> bool {
    use Expression::*;
    match expr {
        Number(..) => number(),
        Call { args, .. } => call(args, environment),
        InfixOp { lhe, rhe, .. } => infix_op(lhe, rhe, environment),
        PrefixOp { rhe, .. } => prefix_op(rhe, environment),
        ParallelOp { rhe, .. } => parallel_op(rhe, environment),
        InlineSwitchOp { cond, if_false, if_true, .. } => {
            inline_switch(cond, if_true, if_false, environment)
        }
        Variable { name, .. } => variable(name, environment),
        ArrayInLine { .. } => array_inline(),
        UniformArray { .. } => uniform_array(),
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}

fn number() -> bool {
    true
}
fn call(params: &[Expression], environment: &Constants) -> bool {
    let mut index = 0;
    let mut constant = true;
    while constant && index < params.len() {
        constant = constant && has_constant_value(&params[index], environment);
        index += 1;
    }
    constant
}
fn array_inline() -> bool {
    false
}
fn uniform_array() -> bool {
    false
}
fn variable(name: &str, environment: &Constants) -> bool {
    *environment.get_variable_or_break(name, file!(), line!())
}

fn inline_switch(
    cond: &Expression,
    if_true: &Expression,
    if_false: &Expression,
    environment: &Constants,
) -> bool {
    has_constant_value(cond, environment)
        && has_constant_value(if_true, environment)
        && has_constant_value(if_false, environment)
}

fn infix_op(lhe: &Expression, rhe: &Expression, environment: &Constants) -> bool {
    has_constant_value(lhe, environment) && has_constant_value(rhe, environment)
}

fn prefix_op(rhe: &Expression, environment: &Constants) -> bool {
    has_constant_value(rhe, environment)
}

fn parallel_op(rhe: &Expression, environment: &Constants) -> bool {
    has_constant_value(rhe, environment)
}

// Set of functions whose purpose is to expand constants
fn expand_statement(stmt: &mut Statement, environment: &mut ExpressionHolder) {
    use Statement::*;
    match stmt {
        IfThenElse { cond, if_case, else_case, .. } => {
            expand_if_then_else(cond, if_case, else_case, environment)
        }
        While { cond, stmt, .. } => expand_while(cond, stmt, environment),
        Return { value, .. } => expand_return(value, environment),
        InitializationBlock { initializations, .. } => {
            expand_initialization_block(initializations, environment)
        }
        Declaration { dimensions, .. } => expand_declaration(dimensions, environment),
        Substitution { access, rhe, .. } => expand_substitution(access, rhe, environment),
        ConstraintEquality { lhe, rhe, .. } => expand_constraint_equality(lhe, rhe, environment),
        LogCall { args, .. } => expand_log_call(args, environment),
        Assert { arg, .. } => expand_assert(arg, environment),
        Block { stmts, .. } => expand_block(stmts, environment),
        MultSubstitution { .. } => unreachable!()
    }
}

fn expand_if_then_else(
    cond: &mut Expression,
    if_case: &mut Statement,
    else_case: &mut Option<Box<Statement>>,
    environment: &mut ExpressionHolder,
) {
    *cond = expand_expression(cond.clone(), environment);
    expand_statement(if_case, environment);
    if let Option::Some(s) = else_case {
        expand_statement(s, environment);
    }
}

fn expand_while(cond: &mut Expression, stmt: &mut Statement, environment: &mut ExpressionHolder) {
    *cond = expand_expression(cond.clone(), environment);
    expand_statement(stmt, environment);
}

fn expand_return(value: &mut Expression, environment: &ExpressionHolder) {
    *value = expand_expression(value.clone(), environment);
}

fn expand_initialization_block(
    initializations: &mut [Statement],
    environment: &mut ExpressionHolder,
) {
    use Statement::{Declaration, Substitution};
    let mut constants = HashSet::new();
    for s in initializations.iter_mut() {
        expand_statement(s, environment);
    }
    for s in initializations.iter_mut() {
        if let Declaration { name, is_constant, .. } = s {
            if *is_constant {
                constants.insert(name.clone());
            }
        }
    }
    for s in initializations.iter_mut() {
        if let Substitution { var, rhe, .. } = s {
            if constants.contains(var) {
                environment.add_variable(var, rhe.clone());
            }
        }
    }
}

fn expand_declaration(dimensions: &mut [Expression], environment: &ExpressionHolder) {
    for d in dimensions {
        *d = expand_expression(d.clone(), environment);
    }
}

fn expand_substitution(
    access: &mut [Access],
    rhe: &mut Expression,
    environment: &ExpressionHolder,
) {
    use Access::ArrayAccess;
    *rhe = expand_expression(rhe.clone(), environment);
    for a in access {
        if let ArrayAccess(e) = a {
            *e = expand_expression(e.clone(), environment);
        }
    }
}

fn expand_constraint_equality(
    lhe: &mut Expression,
    rhe: &mut Expression,
    environment: &ExpressionHolder,
) {
    *lhe = expand_expression(lhe.clone(), environment);
    *rhe = expand_expression(rhe.clone(), environment);
}

fn expand_log_call(args: &mut Vec<LogArgument>, environment: &ExpressionHolder) {
    for arglog in args {
        if let LogArgument::LogExp(arg) = arglog {
            *arg = expand_expression(arg.clone(), environment);
        }
    }
}

fn expand_assert(arg: &mut Expression, environment: &ExpressionHolder) {
    *arg = expand_expression(arg.clone(), environment);
}

fn expand_block(stmts: &mut [Statement], environment: &mut ExpressionHolder) {
    environment.add_variable_block();
    for s in stmts {
        expand_statement(s, environment);
    }
    environment.remove_variable_block();
}

fn expand_expression(expr: Expression, environment: &ExpressionHolder) -> Expression {
    use Expression::*;
    match expr {
        Number(meta, value) => expand_number(meta, value),
        ArrayInLine { meta, values } => expand_array(meta, values, environment),
        UniformArray { meta, value, dimension} => expand_uniform_array(meta, *value, *dimension, environment),
        Call { id, meta, args } => expand_call(id, meta, args, environment),
        InfixOp { meta, lhe, rhe, infix_op } => {
            expand_infix(meta, *lhe, infix_op, *rhe, environment)
        }
        PrefixOp { meta, prefix_op, rhe } => expand_prefix(meta, prefix_op, *rhe, environment),
        ParallelOp { meta, rhe } => expand_parallel(meta, *rhe, environment),
        InlineSwitchOp { meta, cond, if_true, if_false } => {
            expand_inline_switch_op(meta, *cond, *if_true, *if_false, environment)
        }
        Variable { meta, name, access } => expand_variable(meta, name, access, environment),
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}

fn expand_number(meta: Meta, value: BigInt) -> Expression {
    build_number(meta, value)
}

fn expand_array(
    meta: Meta,
    old_values: Vec<Expression>,
    environment: &ExpressionHolder,
) -> Expression {
    let mut values = Vec::new();
    for expr in old_values {
        let new_expression = expand_expression(expr, environment);
        values.push(new_expression);
    }
    build_array_in_line(meta, values)
}

fn expand_uniform_array(
    meta: Meta,
    old_value: Expression,
    old_dimension: Expression,
    environment: &ExpressionHolder,
) -> Expression {
    let value = expand_expression(old_value, environment);
    let dimension = expand_expression(old_dimension, environment);
    build_uniform_array(meta, value, dimension)
}

fn expand_call(
    id: String,
    meta: Meta,
    old_args: Vec<Expression>,
    environment: &ExpressionHolder,
) -> Expression {
    let mut args = Vec::new();
    for expr in old_args {
        let new_expression = expand_expression(expr, environment);
        args.push(new_expression);
    }
    build_call(meta, id, args)
}

fn expand_infix(
    meta: Meta,
    old_lhe: Expression,
    infix_op: ExpressionInfixOpcode,
    old_rhe: Expression,
    environment: &ExpressionHolder,
) -> Expression {
    let lhe = expand_expression(old_lhe, environment);
    let rhe = expand_expression(old_rhe, environment);
    build_infix(meta, lhe, infix_op, rhe)
}

fn expand_prefix(
    meta: Meta,
    prefix_op: ExpressionPrefixOpcode,
    old_rhe: Expression,
    environment: &ExpressionHolder,
) -> Expression {
    let rhe = expand_expression(old_rhe, environment);
    build_prefix(meta, prefix_op, rhe)
}

fn expand_parallel(
    meta: Meta,
    old_rhe: Expression,
    environment: &ExpressionHolder,
) -> Expression {
    let rhe = expand_expression(old_rhe, environment);
    build_parallel_op(meta, rhe)
}

fn expand_inline_switch_op(
    meta: Meta,
    old_cond: Expression,
    old_if_true: Expression,
    old_if_false: Expression,
    environment: &ExpressionHolder,
) -> Expression {
    let cond = expand_expression(old_cond, environment);
    let if_true = expand_expression(old_if_true, environment);
    let if_false = expand_expression(old_if_false, environment);
    build_inline_switch_op(meta, cond, if_true, if_false)
}

fn expand_variable(
    meta: Meta,
    name: String,
    old_access: Vec<Access>,
    environment: &ExpressionHolder,
) -> Expression {
    use Access::*;
    let is_constant = environment.get_variable_res(&name).is_ok();
    if is_constant && old_access.is_empty() {
        let mut expr = environment.get_variable_or_break(&name, file!(), line!()).clone();
        expr.get_mut_meta().change_location(meta.location.clone(), meta.file_id);
        expr
    } else {
        let mut access = Vec::new();
        for a in old_access {
            let new_access = match a {
                ArrayAccess(e) => ArrayAccess(expand_expression(e, environment)),
                component_access => component_access,
            };
            access.push(new_access);
        }
        build_variable(meta, name, access)
    }
}

// Errors
fn broken_invariant_error(meta: &Meta, reports: &mut ReportCollection) {
    let message = "Variable array length".to_string();
    let error_code = ReportCode::NonConstantArrayLength;
    let mut report = Report::error(message, error_code);
    let message = "Non constant expression".to_string();
    report.add_primary(meta.file_location(), meta.get_file_id(), message);
    reports.push(report);
}
