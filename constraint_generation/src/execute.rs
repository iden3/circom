use super::environment_utils::{
    environment::{
        environment_shortcut_add_component, environment_shortcut_add_input,
        environment_shortcut_add_intermediate, environment_shortcut_add_output,
        environment_shortcut_add_variable, ExecutionEnvironment, ExecutionEnvironmentError,
    },
    slice_types::{
        AExpressionSlice, ArithmeticExpression as ArithmeticExpressionGen, ComponentRepresentation,
        ComponentSlice, MemoryError, MemorySlice, SignalSlice, SliceCapacity,
    },
};

use program_structure::constants::UsefulConstants;

use super::execution_data::analysis::Analysis;
use super::execution_data::{ExecutedProgram, ExecutedTemplate, NodePointer};
use super::{
    ast::*, ArithmeticError, FileID, ProgramArchive, Report, ReportCode, ReportCollection
};
use circom_algebra::num_bigint::BigInt;

type AExpr = ArithmeticExpressionGen<String>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum BlockType {
    Known,
    Unknown,
}

struct RuntimeInformation {
    pub block_type: BlockType,
    pub analysis: Analysis,
    pub public_inputs: Vec<String>,
    pub constants: UsefulConstants,
    pub call_trace: Vec<String>,
    pub current_file: FileID,
    pub runtime_errors: ReportCollection,
    pub environment: ExecutionEnvironment,
    pub exec_program: ExecutedProgram,
}
impl RuntimeInformation {
    pub fn new(current_file: FileID, id_max: usize, prime: &String) -> RuntimeInformation {
        RuntimeInformation {
            current_file,
            block_type: BlockType::Known,
            analysis: Analysis::new(id_max),
            public_inputs: vec![],
            constants: UsefulConstants::new(prime),
            call_trace: Vec::new(),
            runtime_errors: ReportCollection::new(),
            environment: ExecutionEnvironment::new(),
            exec_program: ExecutedProgram::new(prime),
        }
    }
}

struct FoldedValue {
    pub arithmetic_slice: Option<AExpressionSlice>,
    pub node_pointer: Option<NodePointer>,
    pub custom_gate_name: Option<String>,
}
impl FoldedValue {
    pub fn valid_arithmetic_slice(f_value: &FoldedValue) -> bool {
        f_value.arithmetic_slice.is_some() && f_value.node_pointer.is_none()
    }
    pub fn valid_node_pointer(f_value: &FoldedValue) -> bool {
        f_value.node_pointer.is_some() && f_value.arithmetic_slice.is_none()
    }
}

impl Default for FoldedValue {
    fn default() -> Self {
        FoldedValue {
            arithmetic_slice: Option::None,
            node_pointer: Option::None,
            custom_gate_name: Option::None
        }
    }
}

enum ExecutionError {
    NonQuadraticConstraint,
    FalseAssert,
}

pub fn constraint_execution(
    program_archive: &ProgramArchive,
    flag_verbose: bool, 
    prime: &String,
) -> Result<ExecutedProgram, ReportCollection> {
    let main_file_id = program_archive.get_file_id_main();
    let mut runtime_information = RuntimeInformation::new(*main_file_id, program_archive.id_max, prime);
    runtime_information.public_inputs = program_archive.get_public_inputs_main_component().clone();
    let folded_value_result = execute_expression(
        program_archive.get_main_expression(),
        program_archive,
        &mut runtime_information,
        flag_verbose
    );
    match folded_value_result {
        Result::Err(_) => Result::Err(runtime_information.runtime_errors),
        Result::Ok(folded_value) => {
            debug_assert!(FoldedValue::valid_node_pointer(&folded_value));
            Result::Ok(runtime_information.exec_program)
        }
    }
}

pub fn execute_constant_expression(
    expression: &Expression,
    program_archive: &ProgramArchive,
    environment: ExecutionEnvironment,
    flag_verbose: bool,
    prime: &String,
) -> Result<BigInt, ReportCollection> {
    let current_file = expression.get_meta().get_file_id();
    let mut runtime_information = RuntimeInformation::new(current_file, program_archive.id_max, prime);
    runtime_information.environment = environment;
    let folded_value_result =
        execute_expression(expression, program_archive, &mut runtime_information, flag_verbose);
    match folded_value_result {
        Result::Err(_) => Result::Err(runtime_information.runtime_errors),
        Result::Ok(folded_value) => {
            debug_assert!(FoldedValue::valid_arithmetic_slice(&folded_value));
            let value = safe_unwrap_to_single_arithmetic_expression(folded_value, line!());
            if let AExpr::Number { value } = value {
                Result::Ok(value)
            } else {
                unreachable!();
            }
        }
    }
}

fn execute_statement(
    stmt: &Statement,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flag_verbose: bool,
) -> Result<Option<FoldedValue>, ()> {
    use Statement::*;
    let id = stmt.get_meta().elem_id;
    Analysis::reached(&mut runtime.analysis, id);
    let res = match stmt {
        InitializationBlock { initializations, .. } => {
            let possible_fold = execute_sequence_of_statements(
                initializations,
                program_archive,
                runtime,
                actual_node,
                flag_verbose
            )?;
            debug_assert!(possible_fold.is_none());
            possible_fold
        }
        Declaration { meta, xtype, name, dimensions, .. } => {
            let mut arithmetic_values = Vec::new();
            for dimension in dimensions.iter() {
                let f_dimensions = execute_expression(dimension, program_archive, runtime, flag_verbose)?;
                arithmetic_values
                    .push(safe_unwrap_to_single_arithmetic_expression(f_dimensions, line!()));
            }
            treat_result_with_memory_error(
                valid_array_declaration(&arithmetic_values),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            let usable_dimensions =
                if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                    dimensions
                } else {
                    unreachable!()
                };
            match xtype {
                VariableType::Component => execute_component_declaration(
                    name,
                    &usable_dimensions,
                    &mut runtime.environment,
                    actual_node,
                ),
                VariableType::Var => environment_shortcut_add_variable(
                    &mut runtime.environment,
                    name,
                    &usable_dimensions,
                ),
                VariableType::Signal(signal_type, _) => execute_signal_declaration(
                    name,
                    &usable_dimensions,
                    *signal_type,
                    &mut runtime.environment,
                    actual_node,
                ),
            }
            Option::None
        }
        Substitution { meta, var, access, op, rhe, .. } => {
            let access_information = treat_accessing(
                meta,
                access,
                program_archive,
                runtime,
                flag_verbose
            )?;
            let r_folded = execute_expression(rhe, program_archive, runtime, flag_verbose)?;
            let possible_constraint = perform_assign(
                meta,
                var,
                &access_information,
                r_folded,
                actual_node,
                runtime
            )?;
            if let (Option::Some(node), AssignOp::AssignConstraintSignal) = (actual_node, op) {
                debug_assert!(possible_constraint.is_some());
                let constrained = possible_constraint.unwrap();
                if constrained.right.is_nonquadratic() {
                    let err = Result::Err(ExecutionError::NonQuadraticConstraint);
                    treat_result_with_execution_error(
                        err,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                } else {
                    let p = runtime.constants.get_p().clone();
                    let symbol = AExpr::Signal { symbol: constrained.left };
                    let expr = AExpr::sub(&symbol, &constrained.right, &p);
                    let ctr = AExpr::transform_expression_to_constraint_form(expr, &p).unwrap();
                    if constrained.custom_gate_name.is_none() {
                        node.add_constraint(ctr);
                    } else {
                        // From a previous semantic analysis we know that in this case we must
                        // have that constrained.right is an AExpr::Signal, so we can safely unwrap
                        // the name of the signal in the right hand side of the expression.
                        debug_assert!(matches!(symbol, AExpr::Signal {..}));
                        debug_assert!(matches!(constrained.right, AExpr::Signal {..}));
                        if let AExpr::Signal { symbol: left } = symbol {
                            if let AExpr::Signal { symbol: right } = constrained.right {
                                let custom_gate_name = constrained.custom_gate_name.unwrap();
                                fn reorder(
                                    left: String,
                                    right: String,
                                    custom_gate_name: &String
                                ) -> (String, String) {
                                    if left.starts_with(custom_gate_name) {
                                        (left, right)
                                    } else {
                                        debug_assert!(right.starts_with(custom_gate_name));
                                        (right, left)
                                    }
                                }

                                // Assignment of the form left <== right
                                let (inner, outer) = reorder(left, right, &custom_gate_name);
                                node.treat_custom_gate_constraint(custom_gate_name, inner, outer);
                            } else {
                                unreachable!();
                            }
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
            Option::None
        }
        ConstraintEquality { meta, lhe, rhe, .. } => {
            debug_assert!(actual_node.is_some());
            let f_left = execute_expression(lhe, program_archive, runtime, flag_verbose)?;
            let f_right = execute_expression(rhe, program_archive, runtime, flag_verbose)?;
            let arith_left = safe_unwrap_to_single_arithmetic_expression(f_left, line!());
            let arith_right = safe_unwrap_to_single_arithmetic_expression(f_right, line!());
            let possible_non_quadratic =
                AExpr::sub(&arith_left, &arith_right, &runtime.constants.get_p());
            if possible_non_quadratic.is_nonquadratic() {
                treat_result_with_execution_error(
                    Result::Err(ExecutionError::NonQuadraticConstraint),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            }
            let quadratic_expression = possible_non_quadratic;
            let constraint_expression = AExpr::transform_expression_to_constraint_form(
                quadratic_expression,
                runtime.constants.get_p(),
            )
            .unwrap();
            if let Option::Some(node) = actual_node {
                node.add_constraint(constraint_expression);
            }
            Option::None
        }
        Return { value, .. } => {
            let mut f_return = execute_expression(value, program_archive, runtime, flag_verbose)?;
            if let Option::Some(slice) = &mut f_return.arithmetic_slice {
                if runtime.block_type == BlockType::Unknown {
                    *slice = AExpressionSlice::new_with_route(slice.route(), &AExpr::NonQuadratic);
                }
            }
            debug_assert!(FoldedValue::valid_arithmetic_slice(&f_return));
            Option::Some(f_return)
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            let else_case = else_case.as_ref().map(|e| e.as_ref());
            let (possible_return, _) = execute_conditional_statement(
                cond,
                if_case,
                else_case,
                program_archive,
                runtime,
                actual_node,
                flag_verbose
            )?;
            possible_return
        }
        While { cond, stmt, .. } => loop {
            let (returned, condition_result) = execute_conditional_statement(
                cond,
                stmt,
                Option::None,
                program_archive,
                runtime,
                actual_node,
                flag_verbose
            )?;
            if returned.is_some() {
                break returned;
            } else if condition_result.is_none() {
                let (returned, _) = execute_conditional_statement(
                    cond,
                    stmt,
                    None,
                    program_archive,
                    runtime,
                    actual_node,
                    flag_verbose
                )?;
                break returned;
            } else if !condition_result.unwrap() {
                break returned;
            }
        },
        Block { stmts, .. } => {
            ExecutionEnvironment::add_variable_block(&mut runtime.environment);
            let return_value =
                execute_sequence_of_statements(stmts, program_archive, runtime, actual_node, flag_verbose)?;
            ExecutionEnvironment::remove_variable_block(&mut runtime.environment);
            return_value
        }
        LogCall { arg, .. } => {
            if flag_verbose{
                let f_result = execute_expression(arg, program_archive, runtime, flag_verbose)?;
                let arith = safe_unwrap_to_single_arithmetic_expression(f_result, line!());
                if AExpr::is_number(&arith){
                    println!("{}", arith);
                }
                else{
                    println!("Unknown")
                }
            }
            Option::None


        }
        Assert { arg, meta, .. } => {
            let f_result = execute_expression(arg, program_archive, runtime, flag_verbose)?;
            let arith = safe_unwrap_to_single_arithmetic_expression(f_result, line!());
            let possible_bool = AExpr::get_boolean_equivalence(&arith, runtime.constants.get_p());
            let result = match possible_bool {
                Some(b) if !b => Err(ExecutionError::FalseAssert),
                _ => Ok(None),
            };
            treat_result_with_execution_error(
                result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        }
    };
    Result::Ok(res)
}

fn execute_expression(
    expr: &Expression,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<FoldedValue, ()> {
    use Expression::*;
    let res = match expr {
        Number(_, value) => {
            let a_value = AExpr::Number { value: value.clone() };
            let ae_slice = AExpressionSlice::new(&a_value);
            FoldedValue { arithmetic_slice: Option::Some(ae_slice), ..FoldedValue::default() }
        }
        Variable { meta, name, access, .. } => {
            if ExecutionEnvironment::has_signal(&runtime.environment, name) {
                execute_signal(meta, name, access, program_archive, runtime, flag_verbose)?
            } else if ExecutionEnvironment::has_component(&runtime.environment, name) {
                execute_component(meta, name, access, program_archive, runtime, flag_verbose)?
            } else if ExecutionEnvironment::has_variable(&runtime.environment, name) {
                execute_variable(meta, name, access, program_archive, runtime, flag_verbose)?
            } else {
                unreachable!();
            }
        }
        ArrayInLine { meta, values, .. } => {
            let mut arithmetic_slice_array = Vec::new();
            for value in values.iter() {
                let f_value = execute_expression(value, program_archive, runtime, flag_verbose)?;
                let slice_value = safe_unwrap_to_arithmetic_slice(f_value, line!());
                arithmetic_slice_array.push(slice_value);
            }
            debug_assert!(!arithmetic_slice_array.is_empty());

            let mut dims = vec![values.len()];
            for dim in arithmetic_slice_array[0].route() {
                dims.push(*dim);
            }
            let mut array_slice = AExpressionSlice::new_with_route(&dims, &AExpr::default());
            let mut row: SliceCapacity = 0;
            while row < arithmetic_slice_array.len() {
                let memory_insert_result = AExpressionSlice::insert_values(
                    &mut array_slice,
                    &[row],
                    &arithmetic_slice_array[row],
                );
                treat_result_with_memory_error(
                    memory_insert_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                row += 1;
            }
            FoldedValue { arithmetic_slice: Option::Some(array_slice), ..FoldedValue::default() }
        }
        InfixOp { meta, lhe, infix_op, rhe, .. } => {
            let l_fold = execute_expression(lhe, program_archive, runtime, flag_verbose)?;
            let r_fold = execute_expression(rhe, program_archive, runtime, flag_verbose)?;
            let l_value = safe_unwrap_to_single_arithmetic_expression(l_fold, line!());
            let r_value = safe_unwrap_to_single_arithmetic_expression(r_fold, line!());
            let r_value = execute_infix_op(meta, *infix_op, &l_value, &r_value, runtime)?;
            let r_slice = AExpressionSlice::new(&r_value);
            FoldedValue { arithmetic_slice: Option::Some(r_slice), ..FoldedValue::default() }
        }
        PrefixOp { prefix_op, rhe, .. } => {
            let folded_value = execute_expression(rhe, program_archive, runtime, flag_verbose)?;
            let arithmetic_value =
                safe_unwrap_to_single_arithmetic_expression(folded_value, line!());
            let arithmetic_result = execute_prefix_op(*prefix_op, &arithmetic_value, runtime)?;
            let slice_result = AExpressionSlice::new(&arithmetic_result);
            FoldedValue { arithmetic_slice: Option::Some(slice_result), ..FoldedValue::default() }
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            let f_cond = execute_expression(cond, program_archive, runtime, flag_verbose)?;
            let ae_cond = safe_unwrap_to_single_arithmetic_expression(f_cond, line!());
            let possible_bool_cond =
                AExpr::get_boolean_equivalence(&ae_cond, runtime.constants.get_p());
            if let Option::Some(bool_cond) = possible_bool_cond {
                if bool_cond {
                    execute_expression(if_true, program_archive, runtime, flag_verbose)?
                } else {
                    execute_expression(if_false, program_archive, runtime, flag_verbose)?
                }
            } else {
                let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
                FoldedValue { arithmetic_slice, ..FoldedValue::default() }
            }
        }
        Call { id, args, .. } => {
            let mut arg_values = Vec::new();
            for arg_expression in args.iter() {
                let f_arg = execute_expression(arg_expression, program_archive, runtime, flag_verbose)?;
                arg_values.push(safe_unwrap_to_arithmetic_slice(f_arg, line!()));
            }
            let new_environment = prepare_environment_for_call(id, &arg_values, program_archive);
            let previous_environment = std::mem::replace(&mut runtime.environment, new_environment);
            let previous_block_type = std::mem::replace(&mut runtime.block_type, BlockType::Known);

            let new_file_id = if program_archive.contains_function(id) {
                program_archive.get_function_data(id).get_file_id()
            } else {
                program_archive.get_template_data(id).get_file_id()
            };
            let previous_id = std::mem::replace(&mut runtime.current_file, new_file_id);

            runtime.call_trace.push(id.clone());
            let folded_result = if program_archive.contains_function(id) {
                execute_function_call(id, program_archive, runtime, flag_verbose)?
            } else {
                execute_template_call(id, &arg_values, program_archive, runtime, flag_verbose)?
            };
            runtime.environment = previous_environment;
            runtime.current_file = previous_id;
            runtime.block_type = previous_block_type;
            runtime.call_trace.pop();
            folded_result
        }
    };
    let expr_id = expr.get_meta().elem_id;
    let res_p = res.arithmetic_slice.clone();
    if let Some(slice) = res_p {
        if slice.is_single() {
            let value = AExpressionSlice::unwrap_to_single(slice);
            Analysis::computed(&mut runtime.analysis, expr_id, value);
        }
    }
    Result::Ok(res)
}

//************************************************* Statement execution support *************************************************

fn execute_component_declaration(
    component_name: &str,
    dimensions: &[SliceCapacity],
    environment: &mut ExecutionEnvironment,
    actual_node: &mut Option<ExecutedTemplate>,
) {
    if let Option::Some(node) = actual_node {
        node.add_component(component_name, dimensions);
        environment_shortcut_add_component(environment, component_name, dimensions);
    } else {
        unreachable!()
    }
}

fn execute_signal_declaration(
    signal_name: &str,
    dimensions: &[SliceCapacity],
    signal_type: SignalType,
    environment: &mut ExecutionEnvironment,
    actual_node: &mut Option<ExecutedTemplate>,
) {
    use SignalType::*;
    if let Option::Some(node) = actual_node {
        node.add_ordered_signal(signal_name, dimensions);
        match signal_type {
            Input => {
                environment_shortcut_add_input(environment, signal_name, dimensions);
                node.add_input(signal_name, dimensions);
            }
            Output => {
                environment_shortcut_add_output(environment, signal_name, dimensions);
                node.add_output(signal_name, dimensions);
            }
            Intermediate => {
                environment_shortcut_add_intermediate(environment, signal_name, dimensions);
                node.add_intermediate(signal_name, dimensions);
            }
        }
    } else {
        unreachable!();
    }
}

/*
    In case the assigment could be a constraint generator the returned value is the constraint
    that will be created
*/
struct Constrained {
    left: String,
    right: AExpr,
    custom_gate_name: Option<String>,
}
fn perform_assign(
    meta: &Meta,
    symbol: &str,
    accessing_information: &AccessingInformation,
    r_folded: FoldedValue,
    actual_node: &mut Option<ExecutedTemplate>,
    runtime: &mut RuntimeInformation,
) -> Result<Option<Constrained>, ()> {
    use super::execution_data::type_definitions::SubComponentData;
    let environment = &mut runtime.environment;
    let full_symbol = create_symbol(symbol, &accessing_information);
    let possible_custom_gate_name = r_folded.custom_gate_name.clone();
    let possible_arithmetic_expression = if ExecutionEnvironment::has_variable(environment, symbol) { // review!
        debug_assert!(accessing_information.signal_access.is_none());
        debug_assert!(accessing_information.after_signal.is_empty());
        let environment_result = ExecutionEnvironment::get_mut_variable_mut(environment, symbol);
        let symbol_content = treat_result_with_environment_error(
            environment_result,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let mut r_slice = safe_unwrap_to_arithmetic_slice(r_folded, line!());
        if runtime.block_type == BlockType::Unknown {
            r_slice = AExpressionSlice::new_with_route(r_slice.route(), &AExpr::NonQuadratic);
        }
        if accessing_information.undefined {
            let new_value =
                AExpressionSlice::new_with_route(symbol_content.route(), &AExpr::NonQuadratic);
            let memory_result =
                AExpressionSlice::insert_values(symbol_content, &vec![], &new_value);
            treat_result_with_memory_error(
                memory_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
        } else {
            let memory_result = AExpressionSlice::insert_values(
                symbol_content,
                &accessing_information.before_signal,
                &r_slice,
            );
            treat_result_with_memory_error(
                memory_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
        }
        Option::None
    } else if ExecutionEnvironment::has_signal(environment, symbol) {
        debug_assert!(accessing_information.signal_access.is_none());
        debug_assert!(accessing_information.after_signal.is_empty());

        let environment_response = ExecutionEnvironment::get_mut_signal_res(environment, symbol);
        let reference_to_signal_content = treat_result_with_environment_error(
            environment_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let memory_response_for_signal_previous_value = SignalSlice::access_values(
            reference_to_signal_content,
            &accessing_information.before_signal,
        );
        let signal_previous_value = treat_result_with_memory_error(
            memory_response_for_signal_previous_value,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        debug_assert!(signal_previous_value.is_single());
        let signal_was_assigned = SignalSlice::unwrap_to_single(signal_previous_value);
        let access_response = if signal_was_assigned {
            Result::Err(MemoryError::AssignmentError)
        } else {
            SignalSlice::insert_values(
                reference_to_signal_content,
                &accessing_information.before_signal,
                &SignalSlice::new(&true),
            )
        };
        treat_result_with_memory_error(
            access_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        Option::Some((safe_unwrap_to_single_arithmetic_expression(r_folded, line!()), None))
    } else if ExecutionEnvironment::has_component(environment, symbol) {
        let environment_response = ExecutionEnvironment::get_mut_component_res(environment, symbol);
        let component_slice = treat_result_with_environment_error(
            environment_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let memory_response = ComponentSlice::get_mut_reference_to_single_value(
            component_slice,
            &accessing_information.before_signal,
        );
        let component = treat_result_with_memory_error(
            memory_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        if accessing_information.signal_access.is_none() {
            debug_assert!(accessing_information.after_signal.is_empty());
            let node_pointer = safe_unwrap_to_valid_node_pointer(r_folded, line!());
            if let Option::Some(actual_node) = actual_node {
                let data = SubComponentData {
                    name: symbol.to_string(),
                    goes_to: node_pointer,
                    indexed_with: accessing_information.before_signal.clone(),
                };
                actual_node.add_arrow(full_symbol.clone(), data);
            } else {
                unreachable!();
            }
            let memory_result = ComponentRepresentation::initialize_component(
                component,
                node_pointer,
                &runtime.exec_program,
            );
            treat_result_with_memory_error(
                memory_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            Option::None
        } else {
            let signal_accessed = accessing_information.signal_access.clone().unwrap();
            debug_assert!(FoldedValue::valid_arithmetic_slice(&r_folded));
            let arithmetic_slice = r_folded.arithmetic_slice.unwrap();
            debug_assert!(arithmetic_slice.is_single());
            let memory_response = ComponentRepresentation::assign_value_to_signal(
                component,
                &signal_accessed,
                &accessing_information.after_signal,
            );
            treat_result_with_memory_error(
                memory_response,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            let custom_gate = if component.is_custom_gate { Some(symbol.to_string()) } else { None };
            Option::Some((AExpressionSlice::unwrap_to_single(arithmetic_slice), custom_gate))
        }
    } else {
        unreachable!();
    };
    if let Option::Some((arithmetic_expression, custom_gate_name)) = possible_arithmetic_expression {
        if custom_gate_name.is_none() {
            let ret = Constrained { 
                left: full_symbol, 
                right: arithmetic_expression, 
                custom_gate_name: possible_custom_gate_name, 
            };
            Result::Ok(Some(ret))
        } else {
            let ret = Constrained {
                left: full_symbol,
                right: arithmetic_expression,
                custom_gate_name
            };
            Result::Ok(Some(ret))
        }
    } else {
        Result::Ok(None)
    }
}

// Evaluates the given condition and executes the corresponding statement. Returns a tuple (a,b) where a is the possible value returned and b is the value of the condition (in case the evaluation was successful)
fn execute_conditional_statement(
    condition: &Expression,
    true_case: &Statement,
    false_case: Option<&Statement>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flag_verbose: bool,
) -> Result<(Option<FoldedValue>, Option<bool>), ()> {
    let f_cond = execute_expression(condition, program_archive, runtime, flag_verbose)?;
    let ae_cond = safe_unwrap_to_single_arithmetic_expression(f_cond, line!());
    let possible_cond_bool_value =
        AExpr::get_boolean_equivalence(&ae_cond, runtime.constants.get_p());
    if let Some(cond_bool_value) = possible_cond_bool_value {
        let ret_value = match false_case {
            Some(else_stmt) if !cond_bool_value => {
                execute_statement(else_stmt, program_archive, runtime, actual_node, flag_verbose)?
            }
            None if !cond_bool_value => None,
            _ => execute_statement(true_case, program_archive, runtime, actual_node, flag_verbose)?,
        };
        Result::Ok((ret_value, Option::Some(cond_bool_value)))
    } else {
        let previous_block_type = runtime.block_type;
        runtime.block_type = BlockType::Unknown;
        let mut ret_value = execute_statement(true_case, program_archive, runtime, actual_node, flag_verbose)?;
        if let Option::Some(else_stmt) = false_case {
            let else_ret = execute_statement(else_stmt, program_archive, runtime, actual_node, flag_verbose)?;
            if ret_value.is_none() {
                ret_value = else_ret;
            }
        }
        runtime.block_type = previous_block_type;
        return Result::Ok((ret_value, Option::None));
    }
}

fn execute_sequence_of_statements(
    stmts: &[Statement],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flag_verbose: bool
) -> Result<Option<FoldedValue>, ()> {
    for stmt in stmts.iter() {
        let f_value = execute_statement(stmt, program_archive, runtime, actual_node, flag_verbose)?;
        if f_value.is_some() {
            return Result::Ok(f_value);
        }
    }
    Result::Ok(Option::None)
}

//************************************************* Expression execution support *************************************************

fn create_symbol(symbol: &str, access_information: &AccessingInformation) -> String {
    let mut appendix = "".to_string();
    let bf_signal = create_index_appendix(&access_information.before_signal);
    let af_signal = create_index_appendix(&access_information.after_signal);
    appendix.push_str(&bf_signal);
    if let Option::Some(signal_accessed) = &access_information.signal_access {
        let signal = format!(".{}", signal_accessed);
        appendix.push_str(&signal);
    }
    appendix.push_str(&af_signal);
    format!("{}{}", symbol, appendix)
}

fn create_index_appendix(indexing: &[usize]) -> String {
    let mut appendix = "".to_string();
    for index in indexing {
        let index = format!("[{}]", index);
        appendix.push_str(&index);
    }
    appendix
}

fn execute_variable(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<FoldedValue, ()> {
    let access_information = treat_accessing(meta, access, program_archive, runtime, flag_verbose)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
        return Result::Ok(FoldedValue { arithmetic_slice, ..FoldedValue::default() });
    }
    debug_assert!(access_information.signal_access.is_none());
    debug_assert!(access_information.after_signal.is_empty());
    let indexing = access_information.before_signal;
    let environment_response = ExecutionEnvironment::get_variable_res(&runtime.environment, symbol);
    let ae_slice = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let memory_response = AExpressionSlice::access_values(&ae_slice, &indexing);
    let ae_slice = treat_result_with_memory_error(
        memory_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    Result::Ok(FoldedValue { arithmetic_slice: Option::Some(ae_slice), ..FoldedValue::default() })
}

fn execute_signal(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<FoldedValue, ()> {
    let access_information = treat_accessing(meta, access, program_archive, runtime, flag_verbose)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
        return Result::Ok(FoldedValue { arithmetic_slice, ..FoldedValue::default() });
    }
    debug_assert!(access_information.signal_access.is_none());
    debug_assert!(access_information.after_signal.is_empty());
    let indexing = &access_information.before_signal;
    let environment_response = if ExecutionEnvironment::has_input(&runtime.environment, symbol) {
        ExecutionEnvironment::get_input_res(&runtime.environment, symbol)
    } else if ExecutionEnvironment::has_output(&runtime.environment, symbol) {
        ExecutionEnvironment::get_output_res(&runtime.environment, symbol)
    } else if ExecutionEnvironment::has_intermediate(&runtime.environment, symbol) {
        ExecutionEnvironment::get_intermediate_res(&runtime.environment, symbol)
    } else {
        unreachable!();
    };
    let signal_slice = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let memory_response = SignalSlice::access_values(signal_slice, indexing);
    let signal_slice = treat_result_with_memory_error(
        memory_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let full_symbol = create_symbol(symbol, &access_information);
    let signal_access = signal_to_arith(full_symbol, signal_slice);
    let arith_slice = treat_result_with_memory_error(
        signal_access,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    Result::Ok(FoldedValue {
        arithmetic_slice: Option::Some(arith_slice),
        ..FoldedValue::default()
    })
}

fn signal_to_arith(symbol: String, slice: SignalSlice) -> Result<AExpressionSlice, MemoryError> {
    let mut expressions = vec![];
    let (route, values) = slice.destruct();
    let mut symbols = vec![];
    unfold_signals(symbol, 0, &route, &mut symbols);
    let mut index = 0;
    while index < symbols.len() && values[index] {
        expressions.push(AExpr::Signal { symbol: symbols[index].clone() });
        index += 1;
    }
    if index == symbols.len() {
        Result::Ok(AExpressionSlice::new_array(route, expressions))
    } else {
        Result::Err(MemoryError::InvalidAccess)
    }
}

fn unfold_signals(current: String, dim: usize, lengths: &[usize], result: &mut Vec<String>) {
    if dim == lengths.len() {
        result.push(current);
    } else {
        for i in 0..lengths[dim] {
            unfold_signals(format!("{}[{}]", current, i), dim + 1, lengths, result)
        }
    }
}

fn execute_component(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<FoldedValue, ()> {
    let access_information = treat_accessing(meta, access, program_archive, runtime, flag_verbose)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
        return Result::Ok(FoldedValue { arithmetic_slice, ..FoldedValue::default() });
    }
    let indexing = &access_information.before_signal;
    let environment_response =
        ExecutionEnvironment::get_component_res(&runtime.environment, symbol);
    let component_slice = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let memory_response = ComponentSlice::access_values(component_slice, indexing);
    let slice_result = treat_result_with_memory_error(
        memory_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let resulting_component = safe_unwrap_to_single(slice_result, line!());
    let read_result = if resulting_component.is_initialized() {
        Result::Ok(resulting_component)
    } else {
        Result::Err(MemoryError::InvalidAccess)
    };
    let checked_component = treat_result_with_memory_error(
        read_result,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let custom_gate_name = if checked_component.is_custom_gate {
        Some(symbol.to_string())
    } else {
        None
    };
    if let Option::Some(signal_name) = &access_information.signal_access {
        let access_after_signal = &access_information.after_signal;
        let signal = treat_result_with_memory_error(
            checked_component.get_signal(signal_name),
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let slice = SignalSlice::access_values(signal, &access_after_signal);
        let slice = treat_result_with_memory_error(
            slice,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let symbol = create_symbol(symbol, &access_information);
        let result = signal_to_arith(symbol, slice).map(|s|
            FoldedValue {
                arithmetic_slice: Option::Some(s),
                custom_gate_name,
                ..FoldedValue::default()
            }
        );
        treat_result_with_memory_error(
            result,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )
    } else {
        Result::Ok(FoldedValue {
            node_pointer: checked_component.node_pointer,
            custom_gate_name,
            ..FoldedValue::default()
        })
    }
}

fn prepare_environment_for_call(
    id: &str,
    arg_values: &[AExpressionSlice],
    program_archive: &ProgramArchive,
) -> ExecutionEnvironment {
    let functions = program_archive.get_function_names();
    let arg_names = if functions.contains(id) {
        program_archive.get_function_data(id).get_name_of_params()
    } else {
        program_archive.get_template_data(id).get_name_of_params()
    };

    let mut environment = ExecutionEnvironment::new();
    debug_assert_eq!(arg_names.len(), arg_values.len());
    for (arg_name, arg_value) in arg_names.iter().zip(arg_values) {
        ExecutionEnvironment::add_variable(&mut environment, arg_name, arg_value.clone());
    }
    environment
}

fn execute_function_call(
    id: &str,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<FoldedValue, ()> {
    let previous_block = runtime.block_type;
    runtime.block_type = BlockType::Known;
    let function_body = program_archive.get_function_data(id).get_body_as_vec();
    let function_result =
        execute_sequence_of_statements(function_body, program_archive, runtime, &mut Option::None, flag_verbose)?;
    runtime.block_type = previous_block;
    let return_value = function_result.unwrap();
    debug_assert!(FoldedValue::valid_arithmetic_slice(&return_value));
    Result::Ok(return_value)
}

fn execute_template_call(
    id: &str,
    parameter_values: &[AExpressionSlice],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<FoldedValue, ()> {
    debug_assert!(runtime.block_type == BlockType::Known);
    let is_main = std::mem::replace(&mut runtime.public_inputs, vec![]);
    let is_parallel = program_archive.get_template_data(id).is_parallel();
    let is_custom_gate = program_archive.get_template_data(id).is_custom_gate();
    let args_names = program_archive.get_template_data(id).get_name_of_params();
    let template_body = program_archive.get_template_data(id).get_body_as_vec();
    let mut args_to_values = vec![];
    let mut instantiation_name = format!("{}(", id);
    for (name, value) in args_names.iter().zip(parameter_values) {
        instantiation_name.push_str(&format!("{},", value.to_string()));
        args_to_values.push((name.clone(), value.clone()));
    }
    if !parameter_values.is_empty() {
        instantiation_name.pop();
    }
    instantiation_name.push(')');
    let existent_node = runtime.exec_program.identify_node(id, &args_to_values);
    let node_pointer = if let Option::Some(pointer) = existent_node {
        pointer
    } else {
        let analysis =
            std::mem::replace(&mut runtime.analysis, Analysis::new(program_archive.id_max));
        let code = program_archive.get_template_data(id).get_body().clone();
        let mut node_wrap = Option::Some(ExecutedTemplate::new(
            is_main,
            id.to_string(),
            instantiation_name,
            args_to_values,
            code,
            is_parallel,
            is_custom_gate,
        ));
        let ret = execute_sequence_of_statements(
            template_body,
            program_archive,
            runtime,
            &mut node_wrap,
            flag_verbose
        )?;
        debug_assert!(ret.is_none());
        let new_node = node_wrap.unwrap();
        let analysis = std::mem::replace(&mut runtime.analysis, analysis);
        let node_pointer = runtime.exec_program.add_node_to_scheme(new_node, analysis);
        node_pointer
    };
    Result::Ok(FoldedValue { node_pointer: Option::Some(node_pointer), ..FoldedValue::default() })
}

fn execute_infix_op(
    meta: &Meta,
    infix: ExpressionInfixOpcode,
    l_value: &AExpr,
    r_value: &AExpr,
    runtime: &mut RuntimeInformation,
) -> Result<AExpr, ()> {
    use ExpressionInfixOpcode::*;
    let field = runtime.constants.get_p();
    let possible_result = match infix {
        Mul => Result::Ok(AExpr::mul(l_value, r_value, field)),
        Div => AExpr::div(l_value, r_value, field),
        Add => Result::Ok(AExpr::add(l_value, r_value, field)),
        Sub => Result::Ok(AExpr::sub(l_value, r_value, field)),
        Pow => Result::Ok(AExpr::pow(l_value, r_value, field)),
        IntDiv => AExpr::idiv(l_value, r_value, field),
        Mod => AExpr::mod_op(l_value, r_value, field),
        ShiftL => AExpr::shift_l(l_value, r_value, field),
        ShiftR => AExpr::shift_r(l_value, r_value, field),
        LesserEq => Result::Ok(AExpr::lesser_eq(l_value, r_value, field)),
        GreaterEq => Result::Ok(AExpr::greater_eq(l_value, r_value, field)),
        Lesser => Result::Ok(AExpr::lesser(l_value, r_value, field)),
        Greater => Result::Ok(AExpr::greater(l_value, r_value, field)),
        Eq => Result::Ok(AExpr::eq(l_value, r_value, field)),
        NotEq => Result::Ok(AExpr::not_eq(l_value, r_value, field)),
        BoolOr => Result::Ok(AExpr::bool_or(l_value, r_value, field)),
        BoolAnd => Result::Ok(AExpr::bool_and(l_value, r_value, field)),
        BitOr => Result::Ok(AExpr::bit_or(l_value, r_value, field)),
        BitAnd => Result::Ok(AExpr::bit_and(l_value, r_value, field)),
        BitXor => Result::Ok(AExpr::bit_xor(l_value, r_value, field)),
    };
    treat_result_with_arithmetic_error(
        possible_result,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )
}

fn execute_prefix_op(
    prefix_op: ExpressionPrefixOpcode,
    value: &AExpr,
    runtime: &mut RuntimeInformation,
) -> Result<AExpr, ()> {
    use ExpressionPrefixOpcode::*;
    let field = runtime.constants.get_p();
    let result = match prefix_op {
        BoolNot => AExpr::not(value, field),
        Sub => AExpr::prefix_sub(value, field),
        Complement => AExpr::complement_256(value, field),
    };
    Result::Ok(result)
}

//************************************************* Indexing support *************************************************

/*
Returns (A,B,C) where:
    A = indexes before a component access as arithmetic expressions
    B = possible signal accessed
    C = index where the signal is accessed, C == access.len() if there is none
*/
fn treat_indexing(
    start: usize,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<(Vec<AExpr>, Option<String>, usize), ()> {
    let mut index_accesses = Vec::new();
    let mut signal_name = Option::None;
    let mut act = start;
    loop {
        if act >= access.len() {
            break;
        }
        match &access[act] {
            Access::ArrayAccess(index) => {
                let index_fold = execute_expression(index, program_archive, runtime, flag_verbose)?;
                let index_arithmetic_expression =
                    safe_unwrap_to_single_arithmetic_expression(index_fold, line!());
                index_accesses.push(index_arithmetic_expression);
                act += 1;
            }
            Access::ComponentAccess(name) => {
                signal_name = Option::Some(name.clone());
                break;
            }
        }
    }
    Result::Ok((index_accesses, signal_name, act))
}

/*
    ae_indexes are a valid indexing when
    all Number values fit in usize
*/
fn valid_indexing(ae_indexes: &[AExpr]) -> Result<(), MemoryError> {
    for ae_index in ae_indexes {
        if ae_index.is_number() && AExpr::get_usize(ae_index).is_none() {
            return Result::Err(MemoryError::OutOfBoundsError);
        }
    }
    Result::Ok(())
}

fn valid_array_declaration(ae_indexes: &[AExpr]) -> Result<(), MemoryError> {
    for ae_index in ae_indexes {
        if !ae_index.is_number() {
            return Result::Err(MemoryError::UnknownSizeDimension);
        }
    }
    Result::Ok(())
}

/*
    ae_indexes Numbers MUST fit in usize,
    this function must be call just
    if valid_indexing does not return
    Result::Err(..)
*/
fn cast_indexing(ae_indexes: &[AExpr]) -> Option<Vec<SliceCapacity>> {
    let mut sc_indexes = Vec::new();
    for ae_index in ae_indexes.iter() {
        if !ae_index.is_number() {
            return Option::None;
        }
        let index = AExpr::get_usize(ae_index).unwrap();
        sc_indexes.push(index);
    }
    Option::Some(sc_indexes)
}

/*
    Usable representation of a series of accesses performed over a symbol.
    AccessingInformation {
        pub undefined: bool ===> true if one of the index values could not be transformed into a SliceCapacity during the process,
        pub before_signal: Vec<SliceCapacity>,
        pub signal_access: Option<String> ==> may not appear,
        pub after_signal: Vec<SliceCapacity>
    }
*/
struct AccessingInformation {
    pub undefined: bool,
    pub before_signal: Vec<SliceCapacity>,
    pub signal_access: Option<String>,
    pub after_signal: Vec<SliceCapacity>,
}
fn treat_accessing(
    meta: &Meta,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flag_verbose: bool
) -> Result<AccessingInformation, ()> {
    let (ae_before_signal, signal_name, signal_index) =
        treat_indexing(0, access, program_archive, runtime, flag_verbose)?;
    let (ae_after_signal, _, _) =
        treat_indexing(signal_index + 1, access, program_archive, runtime, flag_verbose)?;
    treat_result_with_memory_error(
        valid_indexing(&ae_before_signal),
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    treat_result_with_memory_error(
        valid_indexing(&ae_after_signal),
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;

    let possible_before_indexing = cast_indexing(&ae_before_signal);
    let possible_after_indexing = cast_indexing(&ae_after_signal);

    let undefined = possible_before_indexing.is_none() || possible_after_indexing.is_none();
    let signal_access = signal_name;
    let (before_signal, after_signal) = if !undefined {
        (possible_before_indexing.unwrap(), possible_after_indexing.unwrap())
    } else {
        (Vec::new(), Vec::new())
    };
    Result::Ok(AccessingInformation { undefined, before_signal, after_signal, signal_access })
}

//************************************************* Safe transformations *************************************************

fn safe_unwrap_to_single_arithmetic_expression(folded_value: FoldedValue, line: u32) -> AExpr {
    let slice_result = safe_unwrap_to_arithmetic_slice(folded_value, line);
    safe_unwrap_to_single(slice_result, line)
}
fn safe_unwrap_to_arithmetic_slice(folded_value: FoldedValue, line: u32) -> AExpressionSlice {
    debug_assert!(FoldedValue::valid_arithmetic_slice(&folded_value), "Caused by call at {}", line);
    folded_value.arithmetic_slice.unwrap()
}
fn safe_unwrap_to_valid_node_pointer(folded_value: FoldedValue, line: u32) -> NodePointer {
    debug_assert!(FoldedValue::valid_node_pointer(&folded_value), "Caused by call at {}", line);
    folded_value.node_pointer.unwrap()
}
fn safe_unwrap_to_single<C: Clone>(slice: MemorySlice<C>, line: u32) -> C {
    debug_assert!(slice.is_single(), "Caused by call at {}", line);
    MemorySlice::unwrap_to_single(slice)
}

//************************************************* Result handling *************************************************

fn treat_result_with_arithmetic_error<C>(
    arithmetic_error: Result<C, ArithmeticError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ReportCode::RuntimeError;
    match arithmetic_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(arithmetic_error) => {
            let report = match arithmetic_error {
                ArithmeticError::DivisionByZero => {
                    Report::error("Division by zero".to_string(), RuntimeError)
                }
                ArithmeticError::BitOverFlowInShift => {
                    Report::error("Shifting caused bit overflow".to_string(), RuntimeError)
                }
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_memory_error<C>(
    memory_error: Result<C, MemoryError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ReportCode::RuntimeError;
    match memory_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(memory_error) => {
            let report = match memory_error {
                MemoryError::InvalidAccess => {
                    Report::error("Exception caused by invalid access".to_string(), RuntimeError)
                }
                MemoryError::AssignmentError => Report::error(
                    "Exception caused by invalid assignment".to_string(),
                    RuntimeError,
                ),
                MemoryError::OutOfBoundsError => {
                    Report::error("Out of bounds exception".to_string(), RuntimeError)
                }
                MemoryError::UnknownSizeDimension => {
                    Report::error("Array dimension with unknown size".to_string(), RuntimeError)
                }
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_environment_error<C>(
    environment_error: Result<C, ExecutionEnvironmentError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ReportCode::*;
    match environment_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(environment_error) => {
            let report = match environment_error {
                ExecutionEnvironmentError::NonExistentSymbol => {
                    Report::error("Accessing non existent symbol".to_string(), RuntimeError)
                }
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_execution_error<C>(
    execution_error: Result<C, ExecutionError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ExecutionError::*;
    match execution_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(execution_error) => {
            let report = match execution_error {
                NonQuadraticConstraint => Report::error(
                    "Non quadratic constraints are not allowed!".to_string(),
                    ReportCode::RuntimeError,
                ),
                FalseAssert => {
                    Report::error("False assert reached".to_string(), ReportCode::RuntimeError)
                }
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn add_report_to_runtime(
    report: Report,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) {
    let mut report = report;
    report.add_primary(meta.location.clone(), meta.get_file_id(), "found here".to_string());

    let mut trace = "call trace:\n".to_string();
    let mut spacing = "".to_string();
    for call in call_trace.iter() {
        let msg = format!("{}->{}\n", spacing, call);
        trace.push_str(msg.as_str());
        spacing.push_str(" ");
    }
    report.add_note(trace);
    runtime_errors.push(report);
}
