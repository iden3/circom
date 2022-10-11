use program_structure::ast::*;
use program_structure::environment::CircomEnvironment;
use program_structure::function_data::FunctionData;
use program_structure::template_data::TemplateData;

type Environment = CircomEnvironment<(), (), ()>;

pub fn reduce_function(function_data: &mut FunctionData) {
    let mut environment = CircomEnvironment::new();
    for param in function_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = function_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment);
}
pub fn reduce_template(template_data: &mut TemplateData) {
    let mut environment = CircomEnvironment::new();
    for param in template_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = template_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment);
}

fn reduce_types_in_statement(stmt: &mut Statement, environment: &mut Environment) {
    use Statement::*;
    match stmt {
        Substitution { var, access, rhe, meta, .. } => {
            reduce_types_in_substitution(var, access, environment, rhe, meta)
        }
        Declaration { name, xtype, dimensions, .. } => {
            reduce_types_in_declaration(xtype, name, dimensions, environment)
        }
        While { cond, stmt, .. } => reduce_types_in_while(cond, stmt, environment),
        Block { stmts, .. } => reduce_types_in_vec_of_statements(stmts, environment),
        InitializationBlock { initializations, .. } => {
            reduce_types_in_vec_of_statements(initializations, environment)
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            reduce_types_in_conditional(cond, if_case, else_case, environment)
        }
        LogCall { args, .. } => {
                reduce_types_in_log_call(args, environment)
            
        },
        Assert { arg, .. } => reduce_types_in_expression(arg, environment),
        Return { value, .. } => reduce_types_in_expression(value, environment),
        ConstraintEquality { lhe, rhe, .. } => {
            reduce_types_in_constraint_equality(lhe, rhe, environment)
        }
        MultSubstitution { .. } => unreachable!()
    }
}

fn reduce_types_in_log_call(args: &mut Vec<LogArgument>, environment: &Environment){
    for arg in args {
        if let LogArgument::LogExp(exp) = arg {
            reduce_types_in_expression(exp, environment);
        }
    }
}

fn reduce_types_in_expression(expression: &mut Expression, environment: &Environment) {
    use Expression::*;
    match expression {
        Variable { name, access, meta, .. } => {
            reduce_types_in_variable(name, environment, access, meta)
        }
        InfixOp { lhe, rhe, .. } => reduce_types_in_infix(lhe, rhe, environment),
        PrefixOp { rhe, .. } => reduce_types_in_expression(rhe, environment),
        ParallelOp { rhe, .. } => reduce_types_in_expression(rhe, environment),
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            reduce_types_in_inline_switch(cond, if_true, if_false, environment)
        }
        Call { args, .. } => reduce_types_in_vec_of_expressions(args, environment),
        ArrayInLine { values, .. } => reduce_types_in_vec_of_expressions(values, environment),
        UniformArray { value, dimension, .. } => {
            reduce_types_in_expression(value, environment);
            reduce_types_in_expression(dimension, environment);
        }
        Number(..) => {}
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}

fn reduce_types_in_constraint_equality(
    lhe: &mut Expression,
    rhe: &mut Expression,
    environment: &mut Environment,
) {
    reduce_types_in_expression(lhe, environment);
    reduce_types_in_expression(rhe, environment);
}

fn reduce_types_in_declaration(
    xtype: &VariableType,
    name: &str,
    dimensions: &mut [Expression],
    environment: &mut Environment,
) {
    use VariableType::*;
    if *xtype == Var {
        environment.add_variable(name, ());
    } else if *xtype == Component || *xtype == AnonymousComponent {
        environment.add_component(name, ());
    } else {
        environment.add_intermediate(name, ());
    }
    reduce_types_in_vec_of_expressions(dimensions, environment);
}

fn reduce_types_in_substitution(
    name: &str,
    access: &mut [Access],
    environment: &Environment,
    expr: &mut Expression,
    meta: &mut Meta,
) {
    reduce_types_in_variable(name, environment, access, meta);
    reduce_types_in_expression(expr, environment);
}

fn reduce_types_in_while(
    cond: &mut Expression,
    stmt: &mut Statement,
    environment: &mut Environment,
) {
    reduce_types_in_expression(cond, environment);
    reduce_types_in_statement(stmt, environment);
}

fn reduce_types_in_conditional(
    cond: &mut Expression,
    if_branch: &mut Statement,
    else_branch: &mut Option<Box<Statement>>,
    environment: &mut Environment,
) {
    reduce_types_in_expression(cond, environment);
    reduce_types_in_statement(if_branch, environment);
    if let Option::Some(else_stmt) = else_branch {
        reduce_types_in_statement(else_stmt, environment);
    }
}

fn reduce_types_in_vec_of_statements(vec: &mut [Statement], environment: &mut Environment) {
    for stmt in vec {
        reduce_types_in_statement(stmt, environment);
    }
}

fn reduce_types_in_variable(
    name: &str,
    environment: &Environment,
    access: &mut [Access],
    meta: &mut Meta,
) {
    use Access::*;
    use TypeReduction::*;
    let mut reduction = if environment.has_signal(name) {
        Signal
    } else if environment.has_component(name) {
        Component
    } else {
        Variable
    };

    for acc in access {
        if let ArrayAccess(exp) = acc {
            reduce_types_in_expression(exp, environment)
        } else if reduction == Signal{
            reduction = Tag;
        } else {
            reduction = Signal;
        }
    }
    meta.get_mut_type_knowledge().set_reduces_to(reduction);
}

fn reduce_types_in_infix(lhe: &mut Expression, rhe: &mut Expression, environment: &Environment) {
    reduce_types_in_expression(lhe, environment);
    reduce_types_in_expression(rhe, environment);
}

fn reduce_types_in_inline_switch(
    cond: &mut Expression,
    if_true: &mut Expression,
    if_false: &mut Expression,
    environment: &Environment,
) {
    reduce_types_in_expression(cond, environment);
    reduce_types_in_expression(if_true, environment);
    reduce_types_in_expression(if_false, environment);
}

fn reduce_types_in_vec_of_expressions(vec: &mut [Expression], environment: &Environment) {
    for expr in vec {
        reduce_types_in_expression(expr, environment);
    }
}
