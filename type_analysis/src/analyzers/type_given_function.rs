use program_structure::ast::{Expression, Statement};
use program_structure::function_data::FunctionData;
use std::collections::{HashMap, HashSet};

type Type = usize;
type Block = HashMap<String, Type>;
type Environment = Vec<Block>;
type NodeRegister = HashSet<String>;

pub fn type_given_function(
    function_name: &str,
    function_info: &HashMap<String, FunctionData>,
    params_types: &[Type],
) -> Option<Type> {
    let mut explored_functions = NodeRegister::new();
    start(function_name, &mut explored_functions, function_info, params_types)
}

fn add_variable_to_environment(
    _function_name: &str,
    environment: &mut Environment,
    var_name: &str,
    has_type: &Type,
) {
    let last = environment.last_mut().unwrap();
    last.insert(var_name.to_string(), has_type.clone());
}

fn get_type(function_name: &str, environment: &Environment, var_name: &str) -> Type {
    let mut var_type = Option::None;
    for block in environment.iter() {
        if block.get(var_name).is_some() {
            var_type = block.get(var_name);
        }
    }
    match var_type {
        Option::Some(v) => v.clone(),
        Option::None => panic!(
            "in get_type variable {:?} not found in the environment of the function {:?}",
            var_name, function_name
        ),
    }
}

fn start(
    function_name: &str,
    explored_functions: &mut NodeRegister,
    function_info: &HashMap<String, FunctionData>,
    params_types: &[Type],
) -> Option<Type> {
    let function_data = function_info.get(function_name).unwrap();
    let mut environment = Environment::new();
    let mut initial_block = Block::new();
    explored_functions.insert(function_name.to_string());
    for (name, t) in function_data.get_name_of_params().iter().zip(params_types.iter()) {
        initial_block.insert(name.clone(), t.clone());
    }
    environment.push(initial_block);
    look_for_return_value(
        function_name,
        &mut environment,
        explored_functions,
        function_data,
        function_info,
    )
}

fn look_for_return_value(
    function_name: &str,
    environment: &mut Environment,
    explored_functions: &mut NodeRegister,
    function_data: &FunctionData,
    function_info: &HashMap<String, FunctionData>,
) -> Option<Type> {
    let function_body = function_data.get_body_as_vec();
    for stmt in function_body.iter() {
        let ret = look_for_return_in_statement(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            stmt,
        );
        if ret.is_some() {
            return ret;
        }
    }
    Option::None
}
fn look_for_return_in_statement(
    function_name: &str,
    environment: &mut Environment,
    explored_functions: &mut NodeRegister,
    function_data: &FunctionData,
    function_info: &HashMap<String, FunctionData>,
    stmt: &Statement,
) -> Option<Type> {
    match stmt {
        Statement::IfThenElse { if_case, else_case, .. } => {
            let ret1 = look_for_return_in_statement(
                function_name,
                environment,
                explored_functions,
                function_data,
                function_info,
                if_case,
            );
            if ret1.is_some() {
                return ret1;
            }
            let ret2 = match else_case {
                Option::Some(s) => look_for_return_in_statement(
                    function_name,
                    environment,
                    explored_functions,
                    function_data,
                    function_info,
                    s,
                ),
                Option::None => Option::None,
            };
            ret2
        }
        Statement::While { stmt, .. } => look_for_return_in_statement(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            stmt,
        ),
        Statement::Return { value, .. } => look_for_type_in_expression(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            value,
        ),
        Statement::InitializationBlock { initializations, .. } => {
            for initialization in initializations {
                look_for_return_in_statement(
                    function_name,
                    environment,
                    explored_functions,
                    function_data,
                    function_info,
                    initialization,
                );
            }
            Option::None
        }
        Statement::Declaration { name, dimensions, .. } => {
            add_variable_to_environment(function_name, environment, name, &dimensions.len());
            Option::None
        }
        Statement::Block { stmts, .. } => look_for_return_in_block(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            stmts,
        ),
        _ => Option::None,
    }
}

fn look_for_return_in_block(
    function_name: &str,
    environment: &mut Environment,
    explored_functions: &mut NodeRegister,
    function_data: &FunctionData,
    function_info: &HashMap<String, FunctionData>,
    stmts: &Vec<Statement>,
) -> Option<Type> {
    environment.push(Block::new());
    for stmt in stmts.iter() {
        let ret = look_for_return_in_statement(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            stmt,
        );
        if ret.is_some() {
            environment.pop();
            return ret;
        }
    }
    environment.pop();
    Option::None
}

fn look_for_type_in_expression(
    function_name: &str,
    environment: &mut Environment,
    explored_functions: &mut NodeRegister,
    function_data: &FunctionData,
    function_info: &HashMap<String, FunctionData>,
    expression: &Expression,
) -> Option<Type> {
    match expression {
        Expression::InfixOp { lhe, rhe, .. } => {
            let lhe_type = look_for_type_in_expression(
                function_name,
                environment,
                explored_functions,
                function_data,
                function_info,
                lhe,
            );
            if lhe_type.is_some() {
                return lhe_type;
            }
            let rhe_type = look_for_type_in_expression(
                function_name,
                environment,
                explored_functions,
                function_data,
                function_info,
                rhe,
            );
            rhe_type
        }
        Expression::PrefixOp { rhe, .. } => look_for_type_in_expression(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            rhe,
        ),
        Expression::ParallelOp { rhe, .. } => look_for_type_in_expression(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            rhe,
        ),
        Expression::InlineSwitchOp { if_true, if_false, .. } => {
            let if_true_type = look_for_type_in_expression(
                function_name,
                environment,
                explored_functions,
                function_data,
                function_info,
                if_true,
            );
            if if_true_type.is_some() {
                return if_true_type;
            }
            let if_false_type = look_for_type_in_expression(
                function_name,
                environment,
                explored_functions,
                function_data,
                function_info,
                if_false,
            );
            if_false_type
        }
        Expression::Variable { name, access, .. } => {
            let var_type = get_type(function_name, environment, name);
            if access.len() > var_type {
                Option::None
            } else {
                Option::Some(var_type - access.len())
            }
        }
        Expression::Number(..) => Option::Some(0),
        Expression::ArrayInLine { values, .. } => look_for_type_in_expression(
            function_name,
            environment,
            explored_functions,
            function_data,
            function_info,
            &values[0],
        )
        .map(|v| v + 1),
        Expression::UniformArray { value, .. } => {
            let value_type = look_for_type_in_expression(
                function_name,
                environment,
                explored_functions,
                function_data,
                function_info,
                value,
            );
            if value_type.is_some(){
                Option::Some(value_type.unwrap() + 1)
            }
            else{
                None
            }
            
        }
        Expression::Call { id, args, .. } => {
            if explored_functions.contains(id) {
                return Option::None;
            }
            let mut params_types = Vec::new();
            for arg in args {
                let arg_type = look_for_type_in_expression(
                    function_name,
                    environment,
                    explored_functions,
                    function_data,
                    function_info,
                    arg,
                )?;
                params_types.push(arg_type);
            }
            let has_type = start(id, explored_functions, function_info, &params_types);
            has_type
        }
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}
