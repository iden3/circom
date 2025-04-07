use super::environment_utils::{
    environment::{
        environment_shortcut_add_component, environment_shortcut_add_input,
        environment_shortcut_add_intermediate, environment_shortcut_add_output,
        environment_shortcut_add_bus_input, environment_shortcut_add_bus_intermediate,
        environment_shortcut_add_bus_output,
        environment_shortcut_add_variable, ExecutionEnvironment, ExecutionEnvironmentError,
        environment_check_all_components_assigned,
        environment_get_value_tags_bus, environment_get_value_tags_signal
    
    },
    slice_types::{
        AExpressionSlice, ArithmeticExpression as ArithmeticExpressionGen, ComponentRepresentation,
        ComponentSlice, MemoryError, TypeInvalidAccess, TypeAssignmentError, MemorySlice, 
        SignalSlice, SliceCapacity, TagInfo, BusSlice, BusRepresentation,
        FoldedResult, FoldedArgument
    },
};
use program_structure::wire_data::WireType;
use crate::{assignment_utils::*, environment_utils::slice_types::AssignmentState};

use crate::environment_utils::slice_types::BusTagInfo;
use program_structure::constants::UsefulConstants;
use program_structure::bus_data::BusData;
use super::execution_data::analysis::Analysis;
use super::execution_data::{ExecutedBus, ExecutedProgram, ExecutedTemplate, PreExecutedTemplate, NodePointer};
use super::execution_data::type_definitions::{AccessingInformationBus, AccessingInformation, TagNames, TagWire};

use super::{
    ast::*, ArithmeticError, FileID, ProgramArchive, Report, ReportCode, ReportCollection
};
use circom_algebra::num_bigint::BigInt;
use std::collections::{HashMap, BTreeMap};
use crate::FlagsExecution;
type AExpr = ArithmeticExpressionGen<String>;
type AnonymousComponentsInfo = BTreeMap<String, (Meta, Vec<Expression>)>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum BlockType {
    Known,
    Unknown,
}

struct RuntimeInformation {
    pub block_type: BlockType,
    pub conditions_state: Vec<(usize, bool)>,
    pub unknown_counter: usize,
    pub analysis: Analysis,
    pub public_inputs: Vec<String>,
    pub constants: UsefulConstants,
    pub call_trace: Vec<String>,
    pub current_file: FileID,
    pub runtime_errors: ReportCollection,
    pub environment: ExecutionEnvironment,
    pub exec_program: ExecutedProgram,
    pub anonymous_components: AnonymousComponentsInfo,
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
            anonymous_components: AnonymousComponentsInfo::new(),
            conditions_state: Vec::new(),
            unknown_counter: 0,
        }
    }
}

struct FoldedValue {
    pub arithmetic_slice: Option<AExpressionSlice>,
    pub bus_slice: Option<(String, BusSlice)>, // stores the name of the bus and the value
    pub node_pointer: Option<NodePointer>,
    pub bus_node_pointer: Option<NodePointer>,
    pub is_parallel: Option<bool>,
    pub tags: Option<TagWire>,
}
impl FoldedValue {
    pub fn valid_arithmetic_slice(f_value: &FoldedValue) -> bool {
        f_value.arithmetic_slice.is_some() && f_value.node_pointer.is_none() && 
            f_value.is_parallel.is_none() && f_value.bus_node_pointer.is_none()
            && f_value.bus_slice.is_none()
    }
    pub fn valid_bus_slice(f_value: &FoldedValue) -> bool {
        f_value.bus_slice.is_some() && f_value.node_pointer.is_none() && 
            f_value.is_parallel.is_none() && f_value.bus_node_pointer.is_none()
            && f_value.arithmetic_slice.is_none()
    }
    pub fn valid_node_pointer(f_value: &FoldedValue) -> bool {
        f_value.node_pointer.is_some() && f_value.is_parallel.is_some() &&
            f_value.arithmetic_slice.is_none() && f_value.bus_node_pointer.is_none()
            && f_value.bus_slice.is_none()
    }
    pub fn valid_bus_node_pointer(f_value: &FoldedValue) -> bool{
        f_value.bus_node_pointer.is_some() && f_value.node_pointer.is_none() && 
            f_value.is_parallel.is_none() && f_value.arithmetic_slice.is_none()
            && f_value.bus_slice.is_none()
    }
}

impl Default for FoldedValue {
    fn default() -> Self {
        FoldedValue { 
            arithmetic_slice: Option::None, 
            bus_slice: Option::None,
            node_pointer: Option::None, 
            bus_node_pointer: Option::None,
            is_parallel: Option::None, 
            tags: Option::None,
        }
    }
}

enum ExecutionError {
    NonQuadraticConstraint,
    ConstraintInUnknown,
    DeclarationInUnknown,
    TagAssignmentInUnknown,
    UnknownTemplate,
    NonValidTagAssignment,
    FalseAssert,
    ArraySizeTooBig
}

enum ExecutionWarning {
    CanBeQuadraticConstraintSingle(),
    CanBeQuadraticConstraintMultiple(Vec<String>),
}



pub fn constraint_execution(
    program_archive: &ProgramArchive,
    flags: FlagsExecution, 
    prime: &String,
) -> Result<(ExecutedProgram, ReportCollection), ReportCollection> {    
    let main_file_id = program_archive.get_file_id_main();
    let mut runtime_information = RuntimeInformation::new(*main_file_id, program_archive.id_max, prime);
    use Expression::Call;

    runtime_information.public_inputs = program_archive.get_public_inputs_main_component().clone();
    
    let folded_value_result = 
        if let Call { id, args, .. } = &program_archive.get_main_expression() {
            let mut arg_values = Vec::new();
            for arg_expression in args.iter() {
                let f_arg = execute_expression(arg_expression, program_archive, &mut runtime_information, flags);
                arg_values.push(safe_unwrap_to_arithmetic_slice(f_arg.unwrap(), line!()));
                // improve
            }
            execute_template_call_complete(
                id,
                arg_values,
                HashMap::new(),
                program_archive,
                &mut runtime_information,
                flags,
            )
        } else {
            unreachable!("The main expression should be a call."); 
        };
    
    
    match folded_value_result {
        Result::Err(_) => Result::Err(runtime_information.runtime_errors),
        Result::Ok(folded_value) => {
            debug_assert!(FoldedValue::valid_node_pointer(&folded_value));
            Result::Ok((runtime_information.exec_program, runtime_information.runtime_errors))
        }
    }
}

pub fn execute_constant_expression(
    expression: &Expression,
    program_archive: &ProgramArchive,
    environment: ExecutionEnvironment,
    flags: FlagsExecution,
    prime: &String,
) -> Result<BigInt, ReportCollection> {
    let current_file = expression.get_meta().get_file_id();
    let mut runtime_information = RuntimeInformation::new(current_file, program_archive.id_max, prime);
    runtime_information.environment = environment;
    let folded_value_result =
        execute_expression(expression, program_archive, &mut runtime_information, flags);
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

// returns the value and if it can be simplified
fn execute_statement(
    stmt: &Statement,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<(Option<FoldedValue>, bool), ()> {
    use Statement::*;
    let id = stmt.get_meta().elem_id;
    Analysis::reached(&mut runtime.analysis, id);
    let mut can_be_simplified = true;
    let res = match stmt {
        MultSubstitution { .. } => unreachable!(),
        InitializationBlock { initializations, .. } => {
            let (possible_fold, _) = execute_sequence_of_statements(
                initializations,
                program_archive,
                runtime,
                actual_node,
                flags, 
                false
            )?;
            debug_assert!(possible_fold.is_none());
            possible_fold
        }
        Declaration { meta, xtype, name, dimensions, .. } => {
            match xtype {
                VariableType::AnonymousComponent => {
                    if runtime.block_type == BlockType::Unknown{
                        // Case not valid constraint Known/Unknown
                        let err = Result::Err(ExecutionError::DeclarationInUnknown);
                        treat_result_with_execution_error(
                            err,
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                    }
                    execute_anonymous_component_declaration(
                        name,
                        meta.clone(),
                        &dimensions,
                        &mut runtime.environment,
                        &mut runtime.anonymous_components,
                    );
                }
                _ => {
                    let mut arithmetic_values = Vec::new();
                    for dimension in dimensions.iter() {
                        let f_dimensions = 
                            execute_expression(dimension, program_archive, runtime, flags)?;
                        arithmetic_values
                            .push(safe_unwrap_to_single_arithmetic_expression(f_dimensions, line!()));
                    }
                    treat_result_with_memory_error_void(
                        valid_array_declaration(&arithmetic_values),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let usable_dimensions =
                        if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                            dimensions
                        } else {
                            let err = Result::Err(ExecutionError::ArraySizeTooBig);
                            treat_result_with_execution_error(
                                err,
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                            )?
                        };
                    match xtype {
                        VariableType::Component => {
                            if runtime.block_type == BlockType::Unknown{
                                // Case not valid constraint Known/Unknown
                                let err = Result::Err(ExecutionError::DeclarationInUnknown);
                                treat_result_with_execution_error(
                                    err,
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?;
                            }
                            execute_component_declaration(
                                name,
                                &usable_dimensions,
                                &mut runtime.environment,
                                actual_node,
                            )
                        },
                        VariableType::Var => environment_shortcut_add_variable(
                            &mut runtime.environment,
                            name,
                            &usable_dimensions,
                        ),
                        VariableType::Signal(signal_type, tag_list) => {
                            if runtime.block_type == BlockType::Unknown{
                                // Case not valid constraint Known/Unknown
                                let err = Result::Err(ExecutionError::DeclarationInUnknown);
                                treat_result_with_execution_error(
                                    err,
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?;
                            }
                            execute_signal_declaration(
                                name,
                                &usable_dimensions,
                                tag_list,
                                *signal_type,
                                &mut runtime.environment,
                                actual_node,
                            )
                        },
                        VariableType::Bus(_id, signal_type, tag_list) => {
                            if runtime.block_type == BlockType::Unknown{
                                // Case not valid constraint Known/Unknown
                                let err = Result::Err(ExecutionError::DeclarationInUnknown);
                                treat_result_with_execution_error(
                                    err,
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?;
                            }
                            execute_bus_declaration(
                                name,
                                &usable_dimensions,
                                tag_list,
                                *signal_type,
                                &mut runtime.environment,
                                actual_node,
                            )
                        },
                        _ =>{
                            unreachable!()
                        }
                    }

                }
            }
            Option::None
        }
        Substitution { meta, var, access, op, rhe, .. } => {
            let access_information = 
                if ExecutionEnvironment::has_bus(&runtime.environment, var) || ExecutionEnvironment::has_component(&runtime.environment, var){
                    let access_bus = treat_accessing_bus(meta, access, program_archive, runtime, flags)?;
                    TypesAccess{bus_access: Some(access_bus), other_access: None}
                } else{
                    let access_other = treat_accessing(meta, access, program_archive, runtime, flags)?;
                    TypesAccess{bus_access: None, other_access: Some(access_other)}
                };
            
            
            
            let r_folded = execute_expression(rhe, program_archive, runtime, flags)?;
            
            let mut struct_node = if actual_node.is_some(){
                ExecutedStructure::Template(actual_node.as_mut().unwrap())
            } else{
               ExecutedStructure::None
            };
            

            let possible_constraint =
                perform_assign(
                    meta, 
                    var, 
                    *op, 
                    &access_information, 
                    r_folded, 
                    &mut struct_node, 
                    runtime, 
                    program_archive, 
                    flags
                )?;


            if let Option::Some(node) = actual_node {
                if *op == AssignOp::AssignConstraintSignal || (*op == AssignOp::AssignSignal && flags.inspect){
                    debug_assert!(possible_constraint.is_some());
                    
                    if *op == AssignOp::AssignConstraintSignal && runtime.block_type == BlockType::Unknown{
                        // Case not valid constraint Known/Unknown
                        let err = Result::Err(ExecutionError::ConstraintInUnknown);
                        treat_result_with_execution_error(
                            err,
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                    }
                    
                    
                    let constrained = possible_constraint.unwrap();

                    let mut needs_double_arrow = Vec::new();

                    for i in 0..AExpressionSlice::get_number_of_cells(&constrained.right){
                        let value_right = treat_result_with_memory_error(
                            AExpressionSlice::access_value_by_index(&constrained.right, i),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;

                    
                        let signal_left = treat_result_with_memory_error(
                            AExpressionSlice::access_value_by_index(&constrained.left, i),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;

                        if let AssignOp::AssignConstraintSignal = op {
                            if value_right.is_nonquadratic() {
                                let err = Result::Err(ExecutionError::NonQuadraticConstraint);
                                treat_result_with_execution_error(
                                    err,
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?;
                            } else {
                                let p = runtime.constants.get_p().clone();
                                let symbol = signal_left;
                                let expr = AExpr::sub(&symbol, &value_right, &p);
                                let ctr = AExpr::transform_expression_to_constraint_form(expr, &p).unwrap();
                                node.add_constraint(ctr);
                            }
                        } else if let AssignOp::AssignSignal = op {// needs fix, check case arrays
                            //debug_assert!(possible_constraint.is_some());
                            let signal_name = match signal_left{
                                AExpr::Signal { symbol } =>{
                                    symbol
                                },
                                _ => unreachable!()
                            };
                            
                            if !value_right.is_nonquadratic() && !node.is_custom_gate {
                                needs_double_arrow.push(signal_name);
                            }
                        }
                    }

                    if !needs_double_arrow.is_empty() && flags.inspect{
                        // in case we can subsitute the complete expression to ==>
                        if needs_double_arrow.len() == AExpressionSlice::get_number_of_cells(&constrained.right){
                            let err : Result<(),ExecutionWarning> = 
                                Result::Err(ExecutionWarning::CanBeQuadraticConstraintSingle());
                        
                            treat_result_with_execution_warning(
                                err,
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                            )?;
                        } else{
                            let err : Result<(),ExecutionWarning> = 
                                Result::Err(ExecutionWarning::CanBeQuadraticConstraintMultiple(needs_double_arrow));
                        
                            treat_result_with_execution_warning(
                                err,
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                            )?;
                        }
                    }
                }   
            } 
            Option::None
        }
        ConstraintEquality { meta, lhe, rhe, .. } => {
            debug_assert!(actual_node.is_some());

            if runtime.block_type == BlockType::Unknown{
                // Case not valid constraint Known/Unknown
                let err = Result::Err(ExecutionError::ConstraintInUnknown);
                treat_result_with_execution_error(
                    err,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            }

            let f_left = execute_expression(lhe, program_archive, runtime, flags)?;
            let f_right = execute_expression(rhe, program_archive, runtime, flags)?;
            
            let (arith_left, arith_right) = if FoldedValue::valid_arithmetic_slice(&f_left) &&  FoldedValue::valid_arithmetic_slice(&f_right){
                let left = safe_unwrap_to_arithmetic_slice(f_left, line!());
                let right = safe_unwrap_to_arithmetic_slice(f_right, line!());
                let correct_dims_result = AExpressionSlice::check_correct_dims(&left, &Vec::new(), &right, true);
                treat_result_with_memory_error_void(
                    correct_dims_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                (left.destruct().1, right.destruct().1)
            } else if FoldedValue::valid_bus_slice(&f_left) &&  FoldedValue::valid_bus_slice(&f_right){
                let (name_left, slice_left) = safe_unwrap_to_bus_slice(f_left, line!());
                let  (name_right, slice_right) = safe_unwrap_to_bus_slice(f_right, line!());
                
                // Generate an arithmetic slice for the buses left and right
                let mut signals_values_right: Vec<String> = Vec::new();
                let mut signals_values_left: Vec<String> = Vec::new();
                
                // Check that the dimensions of the slices are equal
                let correct_dims_result = BusSlice::check_correct_dims(&slice_left, &Vec::new(), &slice_right, true);
                treat_result_with_memory_error_void(
                    correct_dims_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;

                // Check that the types of the buses are equal 
                // and get the accesses inside the bus
                // We assume that the buses in the slice are all of the same type
                // Generate the arithmetic slices containing the signals
                // Use just the first to generate the bus accesses
                
                let mut inside_bus_signals = Vec::new();
                
                if BusSlice::get_number_of_cells(&slice_left) > 0{
                    let left_i = treat_result_with_memory_error(
                        BusSlice::get_reference_to_single_value_by_index(&slice_left, 0),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let right_i = treat_result_with_memory_error(
                        BusSlice::get_reference_to_single_value_by_index(&slice_right, 0),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    // ensure same type of bus
                    if left_i.node_pointer != right_i.node_pointer{
                        treat_result_with_memory_error(
                            Result::Err(MemoryError::MismatchedInstances),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                    }
                    // generate the inside signals
                    inside_bus_signals = left_i.get_accesses_bus("");

                }

                for i in 0..BusSlice::get_number_of_cells(&slice_left){
                                        
                    let access_index = treat_result_with_memory_error(
                        BusSlice::get_access_index(&slice_left, i),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let string_index = create_index_appendix(&access_index); 

                    for s in &inside_bus_signals{
                        signals_values_right.push(
                            format!(
                                "{}{}{}", name_right, string_index, s.clone()
                        ));
                        signals_values_left.push(
                            format!(
                                "{}{}{}", name_left, string_index, s.clone()
                        ));
                    }
                         
                }

                // Transform the signal names into Arithmetic Expressions
                let mut ae_signals_left = Vec::new();
                for signal_name in signals_values_left{
                    ae_signals_left.push(AExpr::Signal { symbol: signal_name });
                }

                let mut ae_signals_right = Vec::new();
                for signal_name in signals_values_right{
                    ae_signals_right.push(AExpr::Signal { symbol: signal_name });
                }

                (ae_signals_left, ae_signals_right)
            } else{
                unreachable!()
            };

            for i in 0..arith_left.len(){
                let value_left = &arith_left[i];
                let value_right = &arith_right[i];
                let possible_non_quadratic =
                    AExpr::sub(
                        &value_left, 
                        &value_right, 
                        &runtime.constants.get_p()
                    );
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
            }
            Option::None
        }
        Return { value, .. } => {
            let mut f_return = execute_expression(value, program_archive, runtime, flags)?;
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
            let (possible_return, can_simplify, _) = execute_conditional_statement(
                cond,
                if_case,
                else_case,
                program_archive,
                runtime,
                actual_node,
                flags
            )?;
            can_be_simplified = can_simplify;
            possible_return
        }
        While { cond, stmt, .. } => {
            // We update the conditions state of the execution
            runtime.conditions_state.push((runtime.unknown_counter, true));
            runtime.unknown_counter+=1;
            loop {

                let (returned, can_simplify, condition_result) = execute_conditional_statement(
                    cond,
                    stmt,
                    Option::None,
                    program_archive,
                    runtime,
                    actual_node,
                    flags
                )?;
                can_be_simplified &= can_simplify;
                if returned.is_some() {
                    break returned;
                } else if condition_result.is_none() {
                    let (returned, _, _) = execute_conditional_statement(
                        cond,
                        stmt,
                        None,
                        program_archive,
                        runtime,
                        actual_node,
                        flags
                    )?;
                    break returned;
                } else if !condition_result.unwrap() {
                    break returned;
                }
                // We remove the last conditions_state added
                runtime.conditions_state.pop();
            }
        },
        Block { stmts, .. } => {
            ExecutionEnvironment::add_variable_block(&mut runtime.environment);
            let (return_value, can_simplify_block) =
                execute_sequence_of_statements(stmts, program_archive, runtime, actual_node, flags, false)?;
            ExecutionEnvironment::remove_variable_block(&mut runtime.environment);
            can_be_simplified = can_simplify_block;
            return_value
        }
        LogCall { args, .. } => {
            can_be_simplified = false;
            if flags.verbose{
                let mut index = 0;
                for arglog in args {
                    if let LogArgument::LogExp(arg) = arglog{
                        let f_result = execute_expression(arg, program_archive, runtime, flags)?;
                        let arith = safe_unwrap_to_single_arithmetic_expression(f_result, line!());
                        if AExpr::is_number(&arith){
                            print!("{}", arith);
                        }
                        else{
                            print!("Unknown")
                        }
                    }
                    else if let LogArgument::LogStr(s) = arglog {
                            print!("{}",s);
                    }
                    if index != args.len()-1{
                        print!(" ");
                    }
                    index += 1;
                }
                println!("");
            } else{
                for arglog in args {
                    if let LogArgument::LogExp(arg) = arglog{
                        let f_result = execute_expression(arg, program_archive, runtime, flags)?;
                        let _arith = safe_unwrap_to_single_arithmetic_expression(f_result, line!());
                    }
                }
            }
            Option::None
        }
        Assert { arg, meta, .. } => {
            let f_result = execute_expression(arg, program_archive, runtime, flags)?;
            let arith = safe_unwrap_to_single_arithmetic_expression(f_result, line!());
            let possible_bool = AExpr::get_boolean_equivalence(&arith, runtime.constants.get_p());
            let result = match possible_bool {
                Some(b) if !b => Err(ExecutionError::FalseAssert),
                Some(b) if b => Ok(None),
                _ => {
                    can_be_simplified = false;
                    Ok(None)
                }
            };
            treat_result_with_execution_error(
                result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        }
        UnderscoreSubstitution{ meta, rhe, op} =>{
            let f_result = execute_expression(rhe, program_archive, runtime, flags)?;
            if FoldedValue::valid_arithmetic_slice(&f_result){
                let arithmetic_slice = safe_unwrap_to_arithmetic_slice(f_result, line!());
                if *op == AssignOp::AssignConstraintSignal{
                    for i in 0..AExpressionSlice::get_number_of_cells(&arithmetic_slice){
                        let value_cell = treat_result_with_memory_error(
                            AExpressionSlice::access_value_by_index(&arithmetic_slice, i),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                        let constraint_expression = AExpr::transform_expression_to_constraint_form(
                            value_cell,
                            runtime.constants.get_p(),
                        ).unwrap();
                        if let Option::Some(node) = actual_node {
                            for signal in constraint_expression.take_signals(){
                                node.add_underscored_signal(signal);
                            } 
                        }
                    }
                }
            } else if FoldedValue::valid_bus_slice(&f_result){
                let (bus_name, bus_slice) = safe_unwrap_to_bus_slice(f_result, line!());
                let mut signal_values = Vec::new();

                // Get the accesses inside the bus
                // We assume that the buses in the slice are all of the same type
                // Generate the arithmetic slices containing the signals
                // Use just the first to generate the bus accesses
                
                let mut inside_bus_signals = Vec::new();
                
                if BusSlice::get_number_of_cells(&bus_slice) > 0{
                    let left_i = treat_result_with_memory_error(
                        BusSlice::get_reference_to_single_value_by_index(&bus_slice, 0),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    // generate the inside signals
                    inside_bus_signals = left_i.get_accesses_bus("");
                }

                for i in 0..BusSlice::get_number_of_cells(&bus_slice){
                                        
                    let access_index = treat_result_with_memory_error(
                        BusSlice::get_access_index(&bus_slice, i),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let string_index = create_index_appendix(&access_index); 

                    for s in &inside_bus_signals{
                        signal_values.push(
                            format!(
                                "{}{}{}", bus_name, string_index, s.clone()
                        ));
                    }
                         
                }
            
                if let Option::Some(node) = actual_node {
                    for signal_name in &signal_values{
                        node.add_underscored_signal(signal_name);
                    } 
                }
            
            } else{
                unreachable!()
            }
            Option::None
        }
    };
    Result::Ok((res, can_be_simplified))
}

fn execute_bus_statement(
    stmt: &Statement,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut ExecutedBus,
    flags: FlagsExecution
)-> Result<(), ()>{
    use Statement::*;
    let id = stmt.get_meta().elem_id;
    Analysis::reached(&mut runtime.analysis, id);
    let _res = match stmt {
        InitializationBlock { initializations, .. } => {
            execute_sequence_of_bus_statements(
                initializations,
                program_archive,
                runtime,
                actual_node,
                flags, 
            )?
        }
        Declaration { meta, xtype, name, dimensions, .. } => {

            let mut arithmetic_values = Vec::new();
            for dimension in dimensions.iter() {
                let f_dimensions = 
                    execute_expression(dimension, program_archive, runtime, flags)?;
                    arithmetic_values
                    .push(safe_unwrap_to_single_arithmetic_expression(f_dimensions, line!()));
            }
            treat_result_with_memory_error_void(
                valid_array_declaration(&arithmetic_values),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            let usable_dimensions =
            if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                dimensions
            } else {
                let err = Result::Err(ExecutionError::ArraySizeTooBig);
                treat_result_with_execution_error(
                    err,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            };
            match xtype {
    
                VariableType::Signal(_signal_type, tag_list) =>
                    execute_declaration_bus(name, &usable_dimensions, tag_list, &mut runtime.environment, actual_node, false),

                VariableType::Bus(_id, _signal_type, tag_list) =>
                    execute_declaration_bus(name, &usable_dimensions, tag_list, &mut runtime.environment, actual_node, true),
                    
                _ =>{
                    unreachable!()
                }
            }
        }
        Substitution { meta, var, access, op, rhe, .. } => {
            // different access information depending if bus or other variable
            let access_information = 
                if ExecutionEnvironment::has_bus(&runtime.environment, var) || ExecutionEnvironment::has_component(&runtime.environment, var){
                    let access_bus = treat_accessing_bus(meta, access, program_archive, runtime, flags)?;
                    TypesAccess{bus_access: Some(access_bus), other_access: None}
                } else{
                    let access_other = treat_accessing(meta, access, program_archive, runtime, flags)?;
                    TypesAccess{bus_access: None, other_access: Some(access_other)}
                };
            let r_folded = execute_expression(rhe, program_archive, runtime, flags)?;
            let _possible_constraint =
                perform_assign(
                    meta, 
                    var, 
                    *op, 
                    &access_information, 
                    r_folded, 
                    &mut ExecutedStructure::Bus(actual_node), 
                    runtime, 
                    program_archive, 
                    flags
                )?;
        }
        Block { stmts, .. } => {
            execute_sequence_of_bus_statements(stmts, program_archive, runtime, actual_node, flags)?;
        }
        _ => unreachable!(),
    };
    Result::Ok(())
}

fn execute_expression(
    expr: &Expression,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<FoldedValue, ()> {
    use Expression::*;
    let mut can_be_simplified = true;
    let res = match expr {
        Number(_, value) => {
            let a_value = AExpr::Number { value: value.clone() };
            let ae_slice = AExpressionSlice::new(&a_value);
            FoldedValue { arithmetic_slice: Option::Some(ae_slice), ..FoldedValue::default() }
        }
        Variable { meta, name, access, .. } => {
            if ExecutionEnvironment::has_signal(&runtime.environment, name) {
                execute_signal(meta, name, access, program_archive, runtime, flags)?
            } else if ExecutionEnvironment::has_component(&runtime.environment, name) {
                execute_component(meta, name, access, program_archive, runtime, flags)?
            } else if ExecutionEnvironment::has_variable(&runtime.environment, name) {
                execute_variable(meta, name, access, program_archive, runtime, flags)?
            } else if ExecutionEnvironment::has_bus(&runtime.environment, name){
                execute_bus(meta, name, access, program_archive, runtime, flags)?
            }
            else {
                unreachable!();
            }
        }
        ArrayInLine { meta, values, .. } => {
            let mut arithmetic_slice_array = Vec::new();
            for value in values.iter() {
                let f_value = execute_expression(value, program_archive, runtime, flags)?;
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
                    false
                );
                treat_result_with_memory_error_void(
                    memory_insert_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                row += 1;
            }
            FoldedValue { arithmetic_slice: Option::Some(array_slice), ..FoldedValue::default() }
        }
        UniformArray { meta, value, dimension, .. } => {
            let f_dimension = execute_expression(dimension, program_archive, runtime, flags)?;
            let arithmetic_dimension = safe_unwrap_to_single_arithmetic_expression(f_dimension, line!());
            let usable_dimension = if let Option::Some(dimension) = cast_index(&arithmetic_dimension) {
                dimension
            } else {
                unreachable!()
            };

            let f_value = execute_expression(value, program_archive, runtime, flags)?;
            if FoldedValue::valid_arithmetic_slice(&f_value){
                let slice_value = safe_unwrap_to_arithmetic_slice(f_value, line!());
            
                let mut dims = vec![usable_dimension];
                for dim in slice_value.route() {
                    dims.push(*dim);
                }
    
                let mut array_slice = AExpressionSlice::new_with_route(&dims, &AExpr::default());
                let mut row: SliceCapacity = 0;
                while row < usable_dimension {
                    let memory_insert_result = AExpressionSlice::insert_values(
                        &mut array_slice,
                        &[row],
                        &slice_value,
                        false
                    );
                    treat_result_with_memory_error_void(
                        memory_insert_result,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    row += 1;
                }
                FoldedValue { arithmetic_slice: Option::Some(array_slice), ..FoldedValue::default() }
            } else if FoldedValue::valid_bus_node_pointer(&f_value){
                let node_pointer = safe_unwrap_to_valid_bus_node_pointer(f_value, line!());
            
                FoldedValue { bus_node_pointer: Option::Some(node_pointer), ..FoldedValue::default() }
            } else{
                unreachable!();
            }
            
        }
        InfixOp { meta, lhe, infix_op, rhe, .. } => {
            let l_fold = execute_expression(lhe, program_archive, runtime, flags)?;
            let r_fold = execute_expression(rhe, program_archive, runtime, flags)?;
            let l_value = safe_unwrap_to_single_arithmetic_expression(l_fold, line!());
            let r_value = safe_unwrap_to_single_arithmetic_expression(r_fold, line!());
            let r_value = execute_infix_op(meta, *infix_op, &l_value, &r_value, runtime)?;
            let r_slice = AExpressionSlice::new(&r_value);
            FoldedValue { arithmetic_slice: Option::Some(r_slice), ..FoldedValue::default() }
        }
        PrefixOp { prefix_op, rhe, .. } => {
            let folded_value = execute_expression(rhe, program_archive, runtime, flags)?;
            let arithmetic_value =
                safe_unwrap_to_single_arithmetic_expression(folded_value, line!());
            let arithmetic_result = execute_prefix_op(*prefix_op, &arithmetic_value, runtime)?;
            let slice_result = AExpressionSlice::new(&arithmetic_result);
            FoldedValue { arithmetic_slice: Option::Some(slice_result), ..FoldedValue::default() }
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            let f_cond = execute_expression(cond, program_archive, runtime, flags)?;
            let ae_cond = safe_unwrap_to_single_arithmetic_expression(f_cond, line!());
            let possible_bool_cond =
                AExpr::get_boolean_equivalence(&ae_cond, runtime.constants.get_p());
            if let Option::Some(bool_cond) = possible_bool_cond {
                if bool_cond {
                    execute_expression(if_true, program_archive, runtime, flags)?
                } else {
                    execute_expression(if_false, program_archive, runtime, flags)?
                }
            } else {
                let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
                FoldedValue { arithmetic_slice, ..FoldedValue::default() }
            }
        }
        Call { id, args, meta, .. } => {
            let (value, can_simplify) = execute_call(id,meta, args, program_archive, runtime, flags)?;
            can_be_simplified = can_simplify;
            value
        }
        BusCall{id, args, ..} =>{
            let value = execute_bus_call_complete(id, args, program_archive, runtime, flags)?;
            value
        
        }
        ParallelOp{rhe, ..} => {
            let folded_value = execute_expression(rhe, program_archive, runtime, flags)?;
            let (node_pointer, _) =
                safe_unwrap_to_valid_node_pointer(folded_value, line!());
            FoldedValue { node_pointer: Option::Some(node_pointer), is_parallel: Option::Some(true), ..FoldedValue::default() }
        }
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    };
    let expr_id = expr.get_meta().elem_id;
    let res_p = res.arithmetic_slice.clone();
    if let Some(slice) = res_p {
        if slice.is_single() && can_be_simplified{
            let value = AExpressionSlice::unwrap_to_single(slice);
            Analysis::computed(&mut runtime.analysis, expr_id, value);
        }
    }
    Result::Ok(res)
}


//************************************************* Statement execution support *************************************************

fn execute_call(
    id: &String,
    meta: &Meta,
    args: &Vec<Expression>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution,
) -> Result<(FoldedValue, bool), ()> {
    let mut arg_values = Vec::new();

    let is_template = program_archive.contains_template(id);

    for arg_expression in args.iter() {
        let f_arg = execute_expression(arg_expression, program_archive, runtime, flags)?;
        let safe_f_arg = safe_unwrap_to_arithmetic_slice(f_arg, line!());
        if is_template{ // check that all the arguments are known
            for value in MemorySlice::get_reference_values(&safe_f_arg){
                if !AExpr::is_number(&value){
                    treat_result_with_execution_error(
                        Result::Err(ExecutionError::UnknownTemplate),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                }
            }
        }
        arg_values.push(safe_f_arg);
    }
    if program_archive.contains_function(id){ // in this case we execute
        let new_environment = prepare_environment_for_call(id, &arg_values, program_archive);
        let previous_environment = std::mem::replace(&mut runtime.environment, new_environment);
        let previous_block_type = std::mem::replace(&mut runtime.block_type, BlockType::Known);
        let previous_anonymous_components = std::mem::replace(&mut runtime.anonymous_components, AnonymousComponentsInfo::new());

        let new_file_id = program_archive.get_function_data(id).get_file_id();
        let previous_id = std::mem::replace(&mut runtime.current_file, new_file_id);

        runtime.call_trace.push(id.clone());
        let folded_result = execute_function_call(id, program_archive, runtime, flags)?;

        runtime.environment = previous_environment;
        runtime.current_file = previous_id;
        runtime.block_type = previous_block_type;
        runtime.anonymous_components = previous_anonymous_components;
        runtime.call_trace.pop();
        Ok(folded_result)
    } else { // in this case we preexecute and check if it needs tags
        let folded_result = preexecute_template_call(id, &arg_values, program_archive, runtime)?;
        Ok((folded_result, true))
    }
}

fn execute_template_call_complete(
    id: &String,
    arg_values: Vec<AExpressionSlice>,
    tags: HashMap<String, TagWire>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    if program_archive.contains_template(id){ // in this case we execute
        let new_environment = prepare_environment_for_call(id, &arg_values, program_archive);
        let previous_environment = std::mem::replace(&mut runtime.environment, new_environment);
        let previous_block_type = std::mem::replace(&mut runtime.block_type, BlockType::Known);
        let previous_anonymous_components = std::mem::replace(&mut runtime.anonymous_components, AnonymousComponentsInfo::new());

        let new_file_id = program_archive.get_template_data(id).get_file_id();
        let previous_id = std::mem::replace(&mut runtime.current_file, new_file_id);

        runtime.call_trace.push(id.clone());
        let folded_result = execute_template_call(id, arg_values, tags, program_archive, runtime, flags)?;

        runtime.environment = previous_environment;
        runtime.current_file = previous_id;
        runtime.block_type = previous_block_type;
        runtime.anonymous_components = previous_anonymous_components;
        runtime.call_trace.pop();
        Ok(folded_result)
    } else { 
       unreachable!();
    }
}

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

fn execute_anonymous_component_declaration(
    component_name: &str,
    meta: Meta,
    dimensions: &Vec<Expression>,
    environment: &mut ExecutionEnvironment,
    anonymous_components: &mut AnonymousComponentsInfo,
) {
    environment_shortcut_add_component(environment, component_name, &Vec::new());
    anonymous_components.insert(component_name.to_string(), (meta, dimensions.clone()));
}

fn execute_bus_call_complete(
    id: &String,
    args: &Vec<Expression>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    let mut arg_values = Vec::new();
    
    for arg_expression in args.iter() {
        let f_arg = execute_expression(arg_expression, program_archive, runtime, flags)?;
        arg_values.push(safe_unwrap_to_arithmetic_slice(f_arg, line!()));
    }

    if program_archive.contains_bus(id){ // in this case we execute
        let new_environment = prepare_environment_for_call(id, &arg_values, program_archive);
        let previous_environment = std::mem::replace(&mut runtime.environment, new_environment);
        let previous_block_type = std::mem::replace(&mut runtime.block_type, BlockType::Known);
        let previous_anonymous_components = std::mem::replace(&mut runtime.anonymous_components, AnonymousComponentsInfo::new());

        let new_file_id = program_archive.get_bus_data(id).get_file_id();
        let previous_id = std::mem::replace(&mut runtime.current_file, new_file_id);

        runtime.call_trace.push(id.clone());
        let folded_result = execute_bus_call(id, arg_values, program_archive, runtime, flags)?;

        runtime.environment = previous_environment;
        runtime.current_file = previous_id;
        runtime.block_type = previous_block_type;
        runtime.anonymous_components = previous_anonymous_components;
        runtime.call_trace.pop();
        Ok(folded_result)
    } else{
        unreachable!()
    }
}

fn execute_signal_declaration(
    signal_name: &str,
    dimensions: &[SliceCapacity],
    list_tags: &Vec<String>,
    signal_type: SignalType,
    environment: &mut ExecutionEnvironment,
    actual_node: &mut Option<ExecutedTemplate>,
) {
    use SignalType::*;
    let mut tags = TagInfo::new();
    for t in list_tags{
        tags.insert(t.clone(), None);
    } 
    if let Option::Some(node) = actual_node {
        match signal_type {
            Input => {
                if let Some(tags_input) = node.tag_instances().get(signal_name){
                    environment_shortcut_add_input(environment, signal_name, dimensions, &tags_input.tags);
                } else{
                    environment_shortcut_add_input(environment, signal_name, dimensions, &tags);
                }
                node.add_input(signal_name, dimensions, false);
            }
            Output => {
                environment_shortcut_add_output(environment, signal_name, dimensions, &tags);
                node.add_output(signal_name, dimensions, false);
            }
            Intermediate => {
                environment_shortcut_add_intermediate(environment, signal_name, dimensions, &tags);
                node.add_intermediate(signal_name, dimensions, false);
            }
        }
    } else {
        unreachable!();
    }
}

fn execute_declaration_bus(
    signal_name: &str,
    dimensions: &[SliceCapacity],
    list_tags: &Vec<String>,
    environment: &mut ExecutionEnvironment,
    actual_node: &mut ExecutedBus,
    is_bus: bool,
) {
    let mut tags = TagInfo::new();
    for t in list_tags{
        tags.insert(t.clone(), None);
    } 

    if is_bus{
        actual_node.add_bus(signal_name, dimensions, list_tags.clone());
        environment_shortcut_add_bus_intermediate(environment, signal_name, dimensions, &tags);
    } else{
        actual_node.add_signal(signal_name, dimensions, list_tags.clone());
        environment_shortcut_add_intermediate(environment, signal_name, dimensions, &tags);

    }
}

fn execute_bus_declaration(
    bus_name: &str,
    dimensions: &[SliceCapacity],
    list_tags: &Vec<String>,
    signal_type: SignalType,
    environment: &mut ExecutionEnvironment,
    actual_node: &mut Option<ExecutedTemplate>,
) {
    use SignalType::*;
    let mut tags = TagInfo::new();
    for t in list_tags{
        tags.insert(t.clone(), None);
    } 

    if let Option::Some(node) = actual_node {
        match signal_type {
            Input => {
                if let Some(tags_input) = node.tag_instances().get(bus_name){
                    environment_shortcut_add_bus_input(environment, bus_name, dimensions, tags_input);
                } else{
                    
                    let tag_wire = TagWire{
                        tags,
                        fields: None // TODO: FILL THE TAGS
                    };
                    environment_shortcut_add_bus_input(environment, bus_name, dimensions, &tag_wire);
                }
                node.add_input(bus_name, dimensions, true);
            }
            Output => {
                environment_shortcut_add_bus_output(environment, bus_name, dimensions, &tags);
                node.add_output(bus_name, dimensions, true);
            }
            Intermediate => {
                environment_shortcut_add_bus_intermediate(environment, bus_name, dimensions, &tags);
                node.add_intermediate(bus_name, dimensions, true);
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
enum ExecutedStructure<'a>{
    Template(&'a mut ExecutedTemplate),
    Bus(&'a mut ExecutedBus),
    None
}

struct Constrained {
    left: AExpressionSlice,
    right: AExpressionSlice,
}

struct TypesAccess{
    bus_access: Option<AccessingInformationBus>,
    other_access: Option<AccessingInformation>,
}


fn perform_assign(
    meta: &Meta,
    symbol: &str,
    op: AssignOp,
    accessing_information: &TypesAccess,
    r_folded: FoldedValue,
    actual_node: &mut ExecutedStructure,
    runtime: &mut RuntimeInformation,
    program_archive: &ProgramArchive,
    flags: FlagsExecution
) -> Result<Option<Constrained>, ()> {
    use super::execution_data::type_definitions::{SubComponentData, BusData};

    let full_symbol = if accessing_information.bus_access.is_some(){
        create_symbol_bus(symbol, &accessing_information.bus_access.as_ref().unwrap())

    } else{
        create_symbol(symbol, &accessing_information.other_access.as_ref().unwrap())
    };
    let conditions_assignment = if runtime.conditions_state.len() == 0{
        AssignmentState::Assigned(Some(meta.clone()))
    } else{
        AssignmentState::MightAssigned(runtime.conditions_state.clone(), Some(meta.clone()))
    };
    
    let possible_arithmetic_slices = if ExecutionEnvironment::has_variable(&runtime.environment, symbol)
    {
        let accessing_information = accessing_information.other_access.as_ref().unwrap();
        debug_assert!(accessing_information.signal_access.is_none());
        debug_assert!(accessing_information.after_signal.is_empty());
        let environment_result = ExecutionEnvironment::get_mut_variable_mut(&mut runtime.environment, symbol);
        let (symbol_tags, symbol_content) = treat_result_with_environment_error(
            environment_result,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let mut r_tags = if r_folded.tags.is_some(){
            r_folded.tags.as_ref().unwrap().clone()
        } else{
            TagWire::default()
        };
        let mut r_slice = safe_unwrap_to_arithmetic_slice(r_folded, line!());
        if runtime.block_type == BlockType::Unknown {
            r_slice = AExpressionSlice::new_with_route(r_slice.route(), &AExpr::NonQuadratic);
            r_tags = TagWire::default();
        }
        if accessing_information.undefined {
            let new_value =
                AExpressionSlice::new_with_route(symbol_content.route(), &AExpr::NonQuadratic);
            let memory_result =
                AExpressionSlice::insert_values(symbol_content, &vec![], &new_value, false);
            treat_result_with_memory_error_void(
                memory_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            *symbol_tags = TagInfo::new();
        } else {

            let memory_result = AExpressionSlice::insert_values(
                symbol_content,
                &accessing_information.before_signal,
                &r_slice,
                false
            );
            treat_result_with_memory_error_void(
                memory_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            // in case it is a complete assignment assign the tags, if not set the tags to empty
            if AExpressionSlice::get_number_of_cells(symbol_content) == AExpressionSlice::get_number_of_cells(&r_slice){
                *symbol_tags = r_tags.tags;
            } else {
                *symbol_tags = TagInfo::new();
            }
        }
        Option::None
    } else if ExecutionEnvironment::has_signal(&runtime.environment, symbol){
    let accessing_information = accessing_information.other_access.as_ref().unwrap();
    if accessing_information.signal_access.is_some() {
        // it is a tag 
        if ExecutionEnvironment::has_input(&runtime.environment, symbol) {
            treat_result_with_memory_error(
                Result::Err(MemoryError::AssignmentTagInput),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        }

        if runtime.block_type == BlockType::Unknown{
            // Case not valid constraint Known/Unknown
            let err = Result::Err(ExecutionError::TagAssignmentInUnknown);
            treat_result_with_execution_error(
                err,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
        }

        let tag = accessing_information.signal_access.clone().unwrap();
        let environment_response = ExecutionEnvironment::get_mut_signal_res(&mut runtime.environment, symbol);
        let (reference_to_tags, _) = treat_result_with_environment_error(
                environment_response,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
        )?;

        if reference_to_tags.is_init{
            treat_result_with_memory_error(
                Result::Err(MemoryError::AssignmentTagAfterInit),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
        }    
        let arithmetic_slice = r_folded.arithmetic_slice.unwrap();
        let value_aux = AExpressionSlice::unwrap_to_single(arithmetic_slice);
        let value = if let ArithmeticExpressionGen::Number { value } = value_aux {
            value
        } else {
            treat_result_with_execution_error(
                Result::Err(ExecutionError::NonValidTagAssignment),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        };
        let possible_tag = reference_to_tags.tags.get(&tag.clone());
        if let Some(val) = possible_tag {
            if let Some(_) = val {
                treat_result_with_memory_error(
                    Result::Err(MemoryError::AssignmentTagTwice),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            } else { // we add the info saying that the tag is defined
                reference_to_tags.tags.insert(tag.clone(), Option::Some(value.clone()));
                let tag_state = reference_to_tags.definitions.get_mut(&tag).unwrap();
                tag_state.value_defined = true;            
            }
        } else {
            unreachable!()
        } 
               
        Option::None
        
    }else {
        // it is just a signal
        debug_assert!(accessing_information.signal_access.is_none());
        debug_assert!(accessing_information.after_signal.is_empty());

        // to ensure that input signals are not assigned twice, improving error message
        if ExecutionEnvironment::has_input(&runtime.environment, symbol) {
            treat_result_with_memory_error(
                Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentInput(symbol.to_string()))),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        }

        let environment_response = ExecutionEnvironment::get_mut_signal_res(&mut runtime.environment, symbol);
        let (reference_to_tags, reference_to_signal_content) = treat_result_with_environment_error(
            environment_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;

        // Perform the tag assignment
      
        let new_tags = if r_folded.tags.is_some() && op == AssignOp::AssignConstraintSignal{
            r_folded.tags.clone().unwrap()
        } else{
            TagWire::default()
        };
        
        // Perform the tag propagation
        let r_slice = safe_unwrap_to_arithmetic_slice(r_folded, line!());

        if reference_to_tags.remaining_inserts >= MemorySlice::get_number_of_cells(&r_slice){
            reference_to_tags.remaining_inserts -= MemorySlice::get_number_of_cells(&r_slice);
        } else{
            reference_to_tags.remaining_inserts = 0;
        }

        perform_tag_propagation(&mut reference_to_tags.tags, &mut reference_to_tags.definitions, &new_tags.tags, reference_to_tags.is_init);
        reference_to_tags.is_init = true;
        
        // Perform the signal assignment
        let signal_assignment_response = perform_signal_assignment(reference_to_signal_content, &accessing_information.before_signal, &r_slice.route(), &conditions_assignment);
        
        treat_result_with_memory_error_void(
            signal_assignment_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;

        // Get left arithmetic slice
        let mut l_signal_names = Vec::new();
        unfold_signals(full_symbol, 0, r_slice.route(), &mut l_signal_names);
        let mut l_expressions = Vec::new();
        for signal_name in l_signal_names{
            l_expressions.push(AExpr::Signal { symbol: signal_name });
        }
        let l_slice = AExpressionSlice::new_array(r_slice.route().to_vec(), l_expressions);

        // We return both the left and right slices
        Option::Some((l_slice, r_slice))
    }} 
    else if ExecutionEnvironment::has_component(&runtime.environment, symbol) {
        
        let accessing_information = accessing_information.bus_access.as_ref().unwrap();
        
        let environment_response = ExecutionEnvironment::get_mut_component_res(&mut runtime.environment, symbol);
        let component_slice = treat_result_with_environment_error(
            environment_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        
        let is_anonymous_component = runtime.anonymous_components.contains_key(symbol);
        let memory_response = if is_anonymous_component{
            ComponentSlice::get_mut_reference_to_single_value(
                component_slice,
                &Vec::new(),
            )
        } else{
            // in case the component is undef then we do not perform any changes
            // TODO: possible improvement?
            if accessing_information.undefined{
                return Ok(None)
            }
            
            ComponentSlice::get_mut_reference_to_single_value(
                component_slice,
                &accessing_information.array_access,
            )
        };
        
        let component = treat_result_with_memory_error(
            memory_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        
        
        // We distinguish the different cases
        if accessing_information.field_access.is_none() {
            // case complete component assignment
            let (prenode_pointer, is_parallel) = safe_unwrap_to_valid_node_pointer(r_folded, line!());
            let memory_result = ComponentRepresentation::preinitialize_component(
                component,
                is_parallel,
                prenode_pointer,
                &runtime.exec_program,
                is_anonymous_component,
                meta
            );
            treat_result_with_memory_error_void(
                memory_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            if component.is_ready_initialize() {  
                // calls to execute and initialize the component              
                let pretemplate_info = runtime.exec_program.get_prenode_value(prenode_pointer).unwrap();
                let inputs_tags = component.inputs_tags.clone();
                let result = execute_template_call_complete(
                    pretemplate_info.template_name(),
                    pretemplate_info.parameter_instances().clone(),
                    inputs_tags,
                    program_archive,
                    runtime,
                    flags,
                )?;
                let (node_pointer, _is_parallel) = safe_unwrap_to_valid_node_pointer(result, line!());
                
                let environment_response = ExecutionEnvironment::get_mut_component_res(&mut runtime.environment, symbol);
                let component_slice = treat_result_with_environment_error(
                    environment_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                let memory_response = if is_anonymous_component {
                    ComponentSlice::get_mut_reference_to_single_value(
                        component_slice,
                        &Vec::new(),
                    )
                } else{
                    ComponentSlice::get_mut_reference_to_single_value(
                        component_slice,
                        &accessing_information.array_access,
                    )
                };
                let component = treat_result_with_memory_error(
                    memory_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                
                let init_result = ComponentRepresentation::initialize_component(
                    component,
                    node_pointer,
                    &mut runtime.exec_program,
                );
                treat_result_with_memory_error(
                    init_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                        
                match actual_node{
                    ExecutedStructure::Template(node) =>{
                        let data = SubComponentData {
                            name: symbol.to_string(),
                            is_parallel: component.is_parallel,
                            goes_to: node_pointer,
                            indexed_with: accessing_information.array_access.clone(),
                        };
                        node.add_arrow(full_symbol.clone(), data);
                    },
                    ExecutedStructure::Bus(_) =>{
                        unreachable!();
                    },
                    ExecutedStructure::None => {
                        unreachable!();
                    }
                }
            }

            Option::None
        } else {

                let remaining_access = accessing_information.remaining_access.as_ref().unwrap();
                let assigned_ae_slice = 
                    if FoldedValue::valid_arithmetic_slice(&r_folded)
                {

                    // it is signal assignment of a input signal or a field of the bus
                    let signal_accessed = accessing_information.field_access.as_ref().unwrap();
                    let arithmetic_slice = r_folded.arithmetic_slice.unwrap();
                    let tags = if r_folded.tags.is_some() && op == AssignOp::AssignConstraintSignal {
                        r_folded.tags.unwrap()
                    } else {
                        TagWire::default()
                    };
                        
                    let memory_response = if remaining_access.field_access.is_none(){
                        ComponentRepresentation::assign_value_to_signal(
                            component,
                            &signal_accessed,
                            &remaining_access.array_access,
                            &arithmetic_slice.route(),
                            &tags,
                            &conditions_assignment
                            )
                    } else{
                        let aux_slice = SignalSlice::new_with_route(arithmetic_slice.route(), &conditions_assignment);
                        ComponentRepresentation::assign_value_to_bus_field(
                            component,
                            &signal_accessed,
                            &remaining_access,
                            FoldedResult::Signal(aux_slice),
                            &tags,
                            &conditions_assignment
                        )
                    };
                    treat_result_with_memory_error_void(
                        memory_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;

                    // Get left arithmetic slice
                    let mut l_signal_names = Vec::new();
                    unfold_signals(full_symbol, 0, arithmetic_slice.route(), &mut l_signal_names);
                    let mut l_expressions = Vec::new();
                    for signal_name in l_signal_names{
                        l_expressions.push(AExpr::Signal { symbol: signal_name });
                    }
                    let l_slice = AExpressionSlice::new_array(arithmetic_slice.route().to_vec(), l_expressions);

                    (l_slice, arithmetic_slice)
                } else if FoldedValue::valid_bus_slice(&r_folded){
                    // it is a bus input    
                    let bus_accessed = accessing_information.field_access.as_ref().unwrap();
                    let (name_bus, assigned_bus_slice) = r_folded.bus_slice.unwrap();
                    
                    
                    let tags = if r_folded.tags.is_some()  && op == AssignOp::AssignConstraintSignal{
                        r_folded.tags.unwrap()
                    } else {
                        TagWire::default()
                    };

                    // Generate an arithmetic slice for the buses left and right
                    let mut signals_values_right: Vec<String> = Vec::new();
                    let mut signals_values_left: Vec<String> = Vec::new();


                    // Generate the arithmetic slices containing the signals
                    // We assume that the buses in the slice are all of the same type
                    // Use just the first to generate the bus accesses
                    
                    let mut inside_bus_signals = Vec::new();
                
                    if BusSlice::get_number_of_cells(&assigned_bus_slice) > 0{
                        let left_i = treat_result_with_memory_error(
                            BusSlice::get_reference_to_single_value_by_index(&assigned_bus_slice, 0),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                        // generate the inside signals
                        inside_bus_signals = left_i.get_accesses_bus("");
                    }

                    for i in 0..BusSlice::get_number_of_cells(&assigned_bus_slice){
                                        
                        let access_index = treat_result_with_memory_error(
                            BusSlice::get_access_index(&assigned_bus_slice, i),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                        let string_index = create_index_appendix(&access_index); 

                        for s in &inside_bus_signals{
                            signals_values_right.push(
                                format!(
                                    "{}{}{}", name_bus, string_index, s.clone()
                            ));
                            signals_values_left.push(
                                format!(
                                    "{}{}{}", full_symbol, string_index, s.clone()
                            ));
                        }        
                    } 

                    // Transform the signal names into AExpr

                    let mut ae_signals_right = Vec::new();
                    for signal_name in signals_values_right{
                        ae_signals_right.push(AExpr::Signal { symbol: signal_name });
                    }
                    let mut ae_signals_left = Vec::new();
                    for signal_name in signals_values_left{
                        ae_signals_left.push(AExpr::Signal { symbol: signal_name });
                    }
                    
                    let memory_response = 
                        if remaining_access.field_access.is_none()
                    {

                        ComponentRepresentation::assign_value_to_bus(
                            component,
                            &bus_accessed,
                            &remaining_access.array_access,
                            assigned_bus_slice,
                            &tags,
                            &conditions_assignment
                        ) 
                    } else{
                        ComponentRepresentation::assign_value_to_bus_field(
                            component,
                            &bus_accessed,
                            &remaining_access,
                            FoldedResult::Bus(assigned_bus_slice),
                            &tags,
                            &conditions_assignment
                        )
                    };
                    treat_result_with_memory_error_void(
                        memory_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;

                    // Generate the ae expressions

                    let ae_right = AExpressionSlice::new_array([ae_signals_right.len()].to_vec(), ae_signals_right);
                    let ae_left = AExpressionSlice::new_array([ae_signals_left.len()].to_vec(), ae_signals_left);
                    (ae_left, ae_right)

                } else{
                    unreachable!();
                };
                
                if !component.is_initialized && component.is_ready_initialize() {  
                    // calls to execute and initialize the component              
                    let pretemplate_info = runtime.exec_program.get_prenode_value(
                        component.node_pointer.unwrap()
                    ).unwrap();
                    let inputs_tags = component.inputs_tags.clone();
    
                    let folded_result = execute_template_call_complete(
                        pretemplate_info.template_name(),
                        pretemplate_info.parameter_instances().clone(),
                        inputs_tags,
                        program_archive,
                        runtime,
                        flags,
                    )?;
                    
                    let (node_pointer, _is_parallel) = safe_unwrap_to_valid_node_pointer(folded_result, line!());
                    
                    let environment_response = ExecutionEnvironment::get_mut_component_res(&mut runtime.environment, symbol);
                    let component_slice = treat_result_with_environment_error(
                        environment_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let memory_response = if is_anonymous_component {
                        ComponentSlice::get_mut_reference_to_single_value(
                            component_slice,
                            &Vec::new(),
                        )
                    } else{
                        ComponentSlice::get_mut_reference_to_single_value(
                            component_slice,
                            &accessing_information.array_access,
                        )
                    };
                    let component = treat_result_with_memory_error(
                        memory_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    
                    let init_result = ComponentRepresentation::initialize_component(
                        component,
                        node_pointer,
                        &mut runtime.exec_program,
                    );
                    treat_result_with_memory_error_void(
                        init_result,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    match actual_node{
                        ExecutedStructure::Template(node) =>{
                            let data = SubComponentData {
                                name: symbol.to_string(),
                                goes_to: node_pointer,
                                is_parallel: component.is_parallel,
                                indexed_with: accessing_information.array_access.clone(),
                            };
                            let component_symbol = create_component_symbol(symbol, &accessing_information.array_access);
                            node.add_arrow(component_symbol, data);
                        },
                        ExecutedStructure::Bus(_) =>{
                            unreachable!();
                        },
                        ExecutedStructure::None => {
                            unreachable!();
                        }
                    }
                }
                Option::Some(assigned_ae_slice)
            }
    } else if ExecutionEnvironment::has_bus(&runtime.environment, symbol) 
    {

        // we check if it is an input bus, in that case all signals are initialized to true
        let is_input_bus =  ExecutionEnvironment::has_input_bus(&runtime.environment, symbol);
        
        let environment_response = ExecutionEnvironment::get_mut_bus_res(&mut runtime.environment, symbol);
        
        let (tags_info, bus_slice) = treat_result_with_environment_error(
            environment_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;

        let accessing_information = accessing_information.bus_access.as_ref().unwrap();

        // in case the accessing information is undef do not perform changes
        // TODO: possible improvement?

        if accessing_information.undefined{
            return Ok(None)
        }

        if FoldedValue::valid_bus_node_pointer(&r_folded){
            // in this case we are performing an assigment of the type in the node_pointer
            // to the bus in the left
            
            let bus_pointer = r_folded.bus_node_pointer.unwrap();
            // in this case we cannot assign to a single value of the array
            debug_assert!(accessing_information.array_access.len() == 0);
            debug_assert!(accessing_information.field_access.is_none());

            
            for i in 0..BusSlice::get_number_of_cells(&bus_slice){
                let mut value_left = treat_result_with_memory_error(
                    BusSlice::get_mut_reference_to_single_value_by_index(bus_slice, i),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                
                let memory_result = BusRepresentation::initialize_bus(
                    &mut value_left,
                    bus_pointer,
                    &runtime.exec_program,
                    is_input_bus
                );
                treat_result_with_memory_error_void(
                    memory_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;

            }
            let bus_info = runtime.exec_program.get_bus_node(bus_pointer).unwrap();
            
            // Get and update the inside tags of the bus 
            // -> similar to collect_info_tags
            fn collect_info_tags(data_bus: &ExecutedBus, exec_program: &ExecutedProgram, n_inserts: usize)->BTreeMap<String, BusTagInfo>{
                use crate::environment_utils::slice_types::TagState;
                let mut fields = BTreeMap::new();

                for wire_data in &data_bus.fields{
                    let mut tags = BTreeMap::new();
                    let mut definitions = BTreeMap::new();
                    let mut inside_fields = BTreeMap::new();
                    let tag_names = data_bus.tag_names.get(&wire_data.name).unwrap();
                    let size = wire_data.length.iter().fold(1, |aux, val| aux * val);
                    let n_inserts_field = size * n_inserts;
                    for tag in tag_names{
                        tags.insert(tag.clone(), Option::None);
                        definitions.insert(tag.clone(), TagState{
                            defined: true,
                            value_defined: false,
                        });
                    }
                    // in this case it is a bus, add its fields
                    if wire_data.is_bus{
                        let bus_pointer = data_bus.bus_connexions.get(&wire_data.name).unwrap().inspect.goes_to;
                        let data_inside_bus = exec_program.get_bus_node(bus_pointer).unwrap();
                        inside_fields = collect_info_tags(data_inside_bus, exec_program, n_inserts_field);
                    }
                    fields.insert(
                        wire_data.name.clone(),
                        BusTagInfo{
                            tags,
                            definitions,
                            fields: inside_fields,
                            remaining_inserts: n_inserts_field,
                            size,
                            is_init: false,
                        }
                    ); 
                }
                fields
            }
            // only if it is not an input_bus if we are in the main component
            let is_main_component = runtime.call_trace.len() == 1;
            if !is_input_bus || is_main_component{
                let bus_inside_tags = collect_info_tags(bus_info, &runtime.exec_program, BusSlice::get_number_of_cells(&bus_slice));
                tags_info.fields = bus_inside_tags;
            }            
            
            let size = bus_info.size;
            match actual_node{
                ExecutedStructure::Template(node) =>{
                    let data = BusData {
                        name: symbol.to_string(),
                        goes_to: bus_pointer,
                        size,
                    };
                    let component_symbol = create_array_accessed_symbol(symbol, &accessing_information.array_access);
                    node.add_bus_arrow(component_symbol, data);
                },
                ExecutedStructure::Bus(node) =>{
                    let data = BusData {
                        name: symbol.to_string(),
                        goes_to: bus_pointer,
                        size
                    };
                    let component_symbol = create_array_accessed_symbol(symbol, &accessing_information.array_access);
                    node.add_bus_arrow(component_symbol, data);
                },
                ExecutedStructure::None => {
                    unreachable!();
                }
            }
            
            None
        } else if FoldedValue::valid_arithmetic_slice(&r_folded){
            // case assigning a field of the bus
            if meta.get_type_knowledge().is_signal(){

                // in case we are assigning a signal of the complete bus
                // check not valid in input buses
                if is_input_bus {
                    treat_result_with_memory_error(
                        Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentInput(symbol.to_string()))),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?
                }

                let mut value_left = treat_result_with_memory_error(
                    BusSlice::access_values_by_mut_reference(bus_slice, &accessing_information.array_access),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                
                assert!(value_left.len() == 1);
                let single_bus = value_left.get_mut(0).unwrap();
                assert!(accessing_information.field_access.is_some());

                let arithmetic_slice = r_folded.arithmetic_slice.unwrap();
                let tags = if r_folded.tags.is_some()  && op == AssignOp::AssignConstraintSignal{
                    r_folded.tags.unwrap()
                } else {
                    TagWire::default()
                };
    
                let memory_response = single_bus.assign_value_to_field(
                    accessing_information.field_access.as_ref().unwrap(),
                    accessing_information.remaining_access.as_ref().unwrap(),
                    FoldedArgument::Signal(&arithmetic_slice.route().to_vec()),
                    false,
                    &conditions_assignment
                );
                treat_result_with_memory_error_void(
                    memory_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                
                // Perform the tag propagation
                // access to the field that is assigned and then propagate the tags
                let mut to_access = accessing_information;
                let mut tag_data = tags_info;
                while to_access.remaining_access.is_some(){
                    tag_data = tag_data.fields.get_mut(to_access.field_access.as_ref().unwrap()).unwrap();
                    to_access = to_access.remaining_access.as_ref().unwrap();
                }
                perform_tag_propagation_bus(tag_data, &tags, MemorySlice::get_number_of_cells(&arithmetic_slice));

                // Get left arithmetic slice
                let mut l_signal_names = Vec::new();
                unfold_signals(full_symbol, 0, arithmetic_slice.route(), &mut l_signal_names);
                let mut l_expressions = Vec::new();
                for signal_name in l_signal_names{
                    l_expressions.push(AExpr::Signal { symbol: signal_name });
                }
                let l_slice = AExpressionSlice::new_array(arithmetic_slice.route().to_vec(), l_expressions);
                Some((l_slice, arithmetic_slice))
                
            } else if meta.get_type_knowledge().is_tag(){
                // in case we are assigning a tag of the complete bus
                // check not valid in input buses
                if is_input_bus {
                    treat_result_with_memory_error(
                        Result::Err(MemoryError::AssignmentTagInput),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?
                }
                // check not valid in unknown environment
                if runtime.block_type == BlockType::Unknown{
                    // Case not valid constraint Known/Unknown
                    let err = Result::Err(ExecutionError::TagAssignmentInUnknown);
                    treat_result_with_execution_error(
                        err,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                }

                assert!(accessing_information.field_access.is_some());
                let arithmetic_slice = r_folded.arithmetic_slice.unwrap();
                let value_aux = AExpressionSlice::unwrap_to_single(arithmetic_slice);
                let value = if let ArithmeticExpressionGen::Number { value } = value_aux {
                    value
                } else {
                    treat_result_with_execution_error(
                        Result::Err(ExecutionError::NonValidTagAssignment),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?
                };

                let mut ref_to_tags_info = tags_info;
                let mut to_access = accessing_information;
                let mut next_access = accessing_information.remaining_access.as_ref().unwrap();
                while next_access.remaining_access.is_some(){ // it is not the tag access
                    ref_to_tags_info = ref_to_tags_info.fields.get_mut(to_access.field_access.as_ref().unwrap()).unwrap();
                    to_access = to_access.remaining_access.as_ref().unwrap();
                    next_access = next_access.remaining_access.as_ref().unwrap();
                }

                if ref_to_tags_info.is_init{
                    treat_result_with_memory_error(
                        Result::Err(MemoryError::AssignmentTagAfterInit),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?
                }

                let tag_name = to_access.field_access.as_ref().unwrap();
                let possible_tag = ref_to_tags_info.tags.get_mut(tag_name);
                if let Some(val) = possible_tag {
                    if let Some(_) = val {
                        treat_result_with_memory_error(
                            Result::Err(MemoryError::AssignmentTagTwice),
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                        )?
                    } else { // we add the info saying that the tag is defined
                        ref_to_tags_info.tags.insert(tag_name.clone(), Option::Some(value.clone()));
                        let tag_state = ref_to_tags_info.definitions.get_mut(tag_name).unwrap();
                        tag_state.value_defined = true;
                    }
                } 
                None
            } else{
                unreachable!();
            }
        } else if FoldedValue::valid_bus_slice(&r_folded){
            let (name_bus, assigned_bus_slice) = r_folded.bus_slice.as_ref().unwrap();
            // case assigning a bus (complete or field)
            
            // check not valid in input buses
            if is_input_bus {
                treat_result_with_memory_error(
                    Err(MemoryError::AssignmentError(TypeAssignmentError::AssignmentInput(symbol.to_string()))),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            }
            if accessing_information.field_access.is_none(){

                // Perform the tag propagation
                // access to the field that is assigned and then propagate the tags
                let new_tags = r_folded.tags.unwrap();
                
                let mut to_access = accessing_information;
                let mut tag_data = tags_info;
                while to_access.remaining_access.is_some(){
                    tag_data = tag_data.fields.get_mut(to_access.field_access.as_ref().unwrap()).unwrap();
                    to_access = to_access.remaining_access.as_ref().unwrap();
                }
                perform_tag_propagation_bus(tag_data, &new_tags, MemorySlice::get_number_of_cells(&bus_slice));

                // We assign the original buses
                let bus_assignment_response = perform_bus_assignment(bus_slice, &accessing_information.array_access, assigned_bus_slice, false, &conditions_assignment);
                treat_result_with_memory_error_void(
                    bus_assignment_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;

                // Generate an arithmetic slice for the accessed buses
                let mut signals_values_left: Vec<String> = Vec::new();
                let mut signals_values_right = Vec::new();
                
                // We assume that the buses in the slice are all of the same type
                // Use just the first to generate the bus accesses
                
                let mut inside_bus_signals = Vec::new();
                
                if BusSlice::get_number_of_cells(&assigned_bus_slice) > 0{
                    let left_i = treat_result_with_memory_error(
                        BusSlice::get_reference_to_single_value_by_index(&assigned_bus_slice, 0),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    // generate the inside signals
                    inside_bus_signals = left_i.get_accesses_bus("");
                }

                for i in 0..BusSlice::get_number_of_cells(&assigned_bus_slice){
                                        
                    let access_index = treat_result_with_memory_error(
                        BusSlice::get_access_index(&assigned_bus_slice, i),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let string_index = create_index_appendix(&access_index); 

                    for s in &inside_bus_signals{
                        signals_values_right.push(
                            format!(
                                "{}{}{}", name_bus, string_index, s.clone()
                        ));
                        signals_values_left.push(
                                format!(
                                "{}{}{}", full_symbol, string_index, s.clone()
                        ));
                    }        
                } 

                // Transform the signal names into AExpr
                let mut ae_signals_left = Vec::new();
                for signal_name in signals_values_left{
                    ae_signals_left.push(AExpr::Signal { symbol: signal_name });
                }
                let mut ae_signals_right = Vec::new();
                for signal_name in signals_values_right{
                    ae_signals_right.push(AExpr::Signal { symbol: signal_name });
                }

                // Update the left slice
                let l_slice = AExpressionSlice::new_array([ae_signals_left.len()].to_vec(), ae_signals_left);
                let r_slice = AExpressionSlice::new_array([ae_signals_right.len()].to_vec(), ae_signals_right);


                Some((l_slice, r_slice))
            } else{

                let mut value_left = treat_result_with_memory_error(
                    BusSlice::access_values_by_mut_reference(bus_slice, &accessing_information.array_access),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
    
                assert!(value_left.len() == 1);
                let single_bus = value_left.get_mut(0).unwrap();
    
                assert!(accessing_information.field_access.is_some());
                let (name_bus, bus_slice) = r_folded.bus_slice.as_ref().unwrap();
                
                // Perform the tag propagation
                // access to the field that is assigned and then propagate the tags
                let new_tags = r_folded.tags.unwrap();
                
                let mut to_access = accessing_information;
                let mut tag_data = tags_info;
                while to_access.remaining_access.is_some(){
                    tag_data = tag_data.fields.get_mut(to_access.field_access.as_ref().unwrap()).unwrap();
                    to_access = to_access.remaining_access.as_ref().unwrap();
                }
                perform_tag_propagation_bus(tag_data, &new_tags, BusSlice::get_number_of_cells(&bus_slice));
    
                let memory_response = single_bus.assign_value_to_field(
                    accessing_information.field_access.as_ref().unwrap(),
                    accessing_information.remaining_access.as_ref().unwrap(),
                    FoldedArgument::Bus(&bus_slice),
                    false,
                    &conditions_assignment
                );
                treat_result_with_memory_error_void(
                    memory_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;

                // Update the left and right slices
                let mut signals_values_left: Vec<String> = Vec::new();
                let mut signals_values_right: Vec<String> = Vec::new();

                // Generate the arithmetic slices containing the signals
                // We assume that the buses in the slice are all of the same type
                // Use just the first to generate the bus accesses
                    
                let mut inside_bus_signals = Vec::new();
                
                if BusSlice::get_number_of_cells(&assigned_bus_slice) > 0{
                    let left_i = treat_result_with_memory_error(
                        BusSlice::get_reference_to_single_value_by_index(&assigned_bus_slice, 0),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    // generate the inside signals
                    inside_bus_signals = left_i.get_accesses_bus("");
                }

                for i in 0..BusSlice::get_number_of_cells(&assigned_bus_slice){
                                        
                    let access_index = treat_result_with_memory_error(
                        BusSlice::get_access_index(&assigned_bus_slice, i),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let string_index = create_index_appendix(&access_index); 

                    for s in &inside_bus_signals{
                        signals_values_right.push(
                            format!(
                                "{}{}{}", name_bus, string_index, s.clone()
                        ));
                        signals_values_left.push(
                            format!(
                                "{}{}{}", full_symbol, string_index, s.clone()
                        ));
                    }        
                } 

                // Transform the signal names into AExpr

                let mut ae_signals_left = Vec::new();
                for signal_name in signals_values_left{
                    ae_signals_left.push(AExpr::Signal { symbol: signal_name });
                }
                let mut ae_signals_right = Vec::new();
                for signal_name in signals_values_right{
                    ae_signals_right.push(AExpr::Signal { symbol: signal_name });
                }
                let l_slice = AExpressionSlice::new_array([ae_signals_left.len()].to_vec(), ae_signals_left);
                let r_slice = AExpressionSlice::new_array([ae_signals_right.len()].to_vec(), ae_signals_right);
                Some((l_slice, r_slice))                
            }
        } else{

            unreachable!()
        }
        

    } else {
        unreachable!();
    };
    if let Option::Some((arithmetic_slice_left, arithmetic_slice_right)) = possible_arithmetic_slices {
        let ret = Constrained { left: arithmetic_slice_left, right: arithmetic_slice_right };
        Result::Ok(Some(ret))
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
    flags: FlagsExecution,
) -> Result<(Option<FoldedValue>, bool, Option<bool>), ()> {
    let f_cond = execute_expression(condition, program_archive, runtime, flags)?;
    let ae_cond = safe_unwrap_to_single_arithmetic_expression(f_cond, line!());
    let possible_cond_bool_value =
        AExpr::get_boolean_equivalence(&ae_cond, runtime.constants.get_p());
    if let Some(cond_bool_value) = possible_cond_bool_value {
        let (ret_value, can_simplify) = match false_case {
            Option::Some(else_stmt) if !cond_bool_value => {
                execute_statement(else_stmt, program_archive, runtime, actual_node, flags)?
            }
            Option::None if !cond_bool_value => (None, true),
            _ => execute_statement(true_case, program_archive, runtime, actual_node, flags)?,
        };
        Result::Ok((ret_value, can_simplify, Option::Some(cond_bool_value)))
    } else {
        let previous_block_type = runtime.block_type;
        runtime.block_type = BlockType::Unknown;
        // TODO: here instead of executing both branches what we do is to store the values
        // that we assign in each one of the branches and assign later: if we assign in both 
        // of them a signal we return an error. If we assign in just one then we dont return error
        // (maybe a warning indicating that the variable may not get assigned in the if)
        
        
        // We update the conditions state of the execution
        runtime.conditions_state.push((runtime.unknown_counter, true));
        runtime.unknown_counter+=1;

        let (mut ret_value, mut can_simplify) = execute_statement(true_case, program_archive, runtime, actual_node, flags)?;
        if let Option::Some(else_stmt) = false_case {
            // Update the conditions state and set the last to false
            let index = runtime.conditions_state.len()-1;
            runtime.conditions_state[index].1 = false;
            
            let (else_ret, can_simplify_else) = execute_statement(else_stmt, program_archive, runtime, actual_node, flags)?;
            can_simplify &= can_simplify_else;


            // Choose the biggest return value possible

            if ret_value.is_none() {
                ret_value = else_ret;
            } else if ret_value.is_some() && else_ret.is_some(){
                let slice_if = safe_unwrap_to_arithmetic_slice(ret_value.unwrap(),line!());
                let size_if = AExpressionSlice::get_number_of_cells(&slice_if);
                let slice_else = safe_unwrap_to_arithmetic_slice(else_ret.unwrap(),line!());
                let size_else  =  AExpressionSlice::get_number_of_cells(&slice_else);
                if size_else > size_if{
                    ret_value = Some(FoldedValue{
                        arithmetic_slice: Some(slice_else), 
                        ..FoldedValue::default()
                    });
                } else{
                    ret_value = Some(FoldedValue{
                        arithmetic_slice: Some(slice_if), 
                        ..FoldedValue::default()
                    });
                }

            }
        }
        // remove the last condition added
        runtime.conditions_state.pop();
        runtime.block_type = previous_block_type;
        return Result::Ok((ret_value, can_simplify, Option::None));
    }
}

fn execute_sequence_of_statements(
    stmts: &[Statement],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
    is_complete_template: bool
) -> Result<(Option<FoldedValue>, bool), ()> {
    let mut can_be_simplified = true;
    for stmt in stmts.iter() {
        let (f_value, can_simplify) = execute_statement(stmt, program_archive, runtime, actual_node, flags)?;
        can_be_simplified &= can_simplify;
        if f_value.is_some() {
            return Result::Ok((f_value, can_be_simplified));
        }
    }
    if is_complete_template{
        execute_delayed_declarations(program_archive, runtime, actual_node, flags)?;
    }
    Result::Ok((Option::None, can_be_simplified))
}

fn execute_sequence_of_bus_statements(
    stmts: &[Statement],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut ExecutedBus,
    flags: FlagsExecution,
) -> Result<(), ()> {
    for stmt in stmts.iter() {
        execute_bus_statement(stmt, program_archive, runtime, actual_node, flags)?;
    }
    Result::Ok(())
}

fn execute_delayed_declarations(
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
)-> Result<(), ()> {
    for (component_name, (meta, dimensions)) in runtime.anonymous_components.clone(){
        let mut arithmetic_values = Vec::new();
        for dimension in dimensions.iter() {
            let f_dimensions = execute_expression(dimension, program_archive, runtime, flags)?;
            arithmetic_values
                .push(safe_unwrap_to_single_arithmetic_expression(f_dimensions, line!()));
        }
        treat_result_with_memory_error_void(
            valid_array_declaration(&arithmetic_values),
            &meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let usable_dimensions =
            if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                dimensions
            } else {
                let err = Result::Err(ExecutionError::ArraySizeTooBig);
                treat_result_with_execution_error(
                    err,
                    &meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            };
        if let Option::Some(node) = actual_node {
            node.add_component(&component_name, &usable_dimensions);
        }           
    }
    Result::Ok(())
}

//************************************************* Expression execution support *************************************************

fn create_array_accessed_symbol(symbol: &str, array_access: &Vec<usize>) -> String {
    let mut appendix = "".to_string();
    let access = create_index_appendix(array_access);
    appendix.push_str(&access);
    format!("{}{}", symbol, appendix)
}
fn create_component_symbol(symbol: &str, access_information: &Vec<usize>) -> String {
    create_array_accessed_symbol(symbol, &access_information)
}

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

fn create_symbol_bus(symbol: &str, access_information: &AccessingInformationBus) -> String {
    let mut appendix = symbol.to_string();
    let bf_field = create_index_appendix(&access_information.array_access);
    appendix.push_str(&bf_field);
    if let Option::Some(field) = &access_information.field_access {
        let field = format!(".{}", field);
        appendix.push_str(&field);
    }
    if let Option::Some(after_field) = &access_information.remaining_access {
        create_symbol_bus(&appendix, after_field)
    } else{
        appendix
    }
}


fn create_index_appendix(indexing: &[usize]) -> String {
    let mut appendix = "".to_string();
    for index in indexing {
        let index = format!("[{}]", index);
        appendix.push_str(&index);
    }
    appendix
}


// fn create_symbols_form_access_bus(symbol: &str, access_information: &AccessingInformationBus,runtime: &RuntimeInformation)-> Vec<String>{
//     let prefix = create_symbol_bus(symbol, access_information);
//     if ExecutionEnvironment::has_bus(&runtime.environment, symbol) {
//         execute_signal(meta, name, access, program_archive, runtime, flags)?
//     } 
// }

fn execute_variable(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<FoldedValue, ()> {
    let access_information = treat_accessing(meta, access, program_archive, runtime, flags)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
        return Result::Ok(FoldedValue { arithmetic_slice, ..FoldedValue::default() });
    }
    debug_assert!(access_information.signal_access.is_none());
    debug_assert!(access_information.after_signal.is_empty());
    let indexing = access_information.before_signal;
    let environment_response = ExecutionEnvironment::get_variable_res(&runtime.environment, symbol);
    let (var_tag, ae_slice) = treat_result_with_environment_error(
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
    let tags = TagWire{
        tags: var_tag.clone(),
        fields: None
    };
    Result::Ok(FoldedValue { arithmetic_slice: Option::Some(ae_slice), tags: Option::Some(tags), ..FoldedValue::default() })
}

fn execute_signal(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<FoldedValue, ()> {
    let access_information = treat_accessing(meta, access, program_archive, runtime, flags)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
        return Result::Ok(FoldedValue { arithmetic_slice, ..FoldedValue::default() });
    }
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
    let (tag_data,  signal_slice) = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    if let Some(acc) = access_information.signal_access {
        if tag_data.tags.contains_key(&acc) {
            let value_tag = tag_data.tags.get(&acc).unwrap();
            let state = tag_data.definitions.get(&acc).unwrap();
            if let Some(value_tag) = value_tag { // tag has value
                // access only allowed when (1) it is value defined by user or (2) it is completely assigned
                if state.value_defined || tag_data.remaining_inserts == 0{
                    let a_value = AExpr::Number { value: value_tag.clone() };
                    let ae_slice = AExpressionSlice::new(&a_value);
                    Result::Ok(FoldedValue { arithmetic_slice: Option::Some(ae_slice), ..FoldedValue::default() })
                } else{
                    let error = MemoryError::TagValueNotInitializedAccess;
                    treat_result_with_memory_error(
                        Result::Err(error),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?
                }
                
            }
            else {
                let error = MemoryError::TagValueNotInitializedAccess;
                treat_result_with_memory_error(
                    Result::Err(error),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            }
        }
        else {
             unreachable!() 
        }
    } else {
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

        // check which tags are propagated
        let tags_propagated = compute_propagated_tags(&tag_data.tags, &tag_data.definitions, tag_data.remaining_inserts);
        let tags = TagWire{
            tags: tags_propagated,
            fields: None
        };
        Result::Ok(FoldedValue {
            arithmetic_slice: Option::Some(arith_slice),
            tags: Option::Some(tags),
            ..FoldedValue::default()
        })
    }
}

fn signal_to_arith(symbol: String, slice: SignalSlice) -> Result<AExpressionSlice, MemoryError> {
    let mut expressions = vec![];
    let (route, values) = slice.destruct();
    let mut symbols = vec![];
    unfold_signals(symbol, 0, &route, &mut symbols);
    let mut index = 0;
    while index < symbols.len(){
        match values[index]{
            AssignmentState::NoAssigned =>{
                return Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedSignal));
            }
            _ => {

            }
        }
        expressions.push(AExpr::Signal { symbol: symbols[index].clone() });
        index += 1;
    }
    if index == symbols.len() {
        Result::Ok(AExpressionSlice::new_array(route, expressions))
    } else {
        Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedSignal))
    }
    // TODO: in case inspect return warning in case might assigned?
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

fn execute_bus(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<FoldedValue, ()> {
    let access_information = treat_accessing_bus(meta, access, program_archive, runtime, flags)?;
    
    let is_tag = match meta.get_type_knowledge().get_reduces_to(){
        TypeReduction::Tag => true,
        _ => false
    };

    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
        return Result::Ok(FoldedValue { arithmetic_slice, ..FoldedValue::default() });
    }
    let environment_response =
        ExecutionEnvironment::get_bus_res(&runtime.environment, symbol);
    let (tag_data, bus_slice) = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;

    let memory_response = BusSlice::access_values(&bus_slice, &access_information.array_access);
    let bus_slice = treat_result_with_memory_error(
        memory_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;

    if access_information.field_access.is_none() {

        // Case we are accessing the complete bus or array of buses
        let symbol = create_symbol_bus(symbol, &access_information);

        // Compute which tags are propagated 
        let tags_propagated = compute_propagated_tags_bus(&tag_data);

        // Check that all the buses are completely assigned

        for i in 0..BusSlice::get_number_of_cells(&bus_slice){
            let value_left = treat_result_with_memory_error(
                BusSlice::get_reference_to_single_value_by_index(&bus_slice, i),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            
            if value_left.has_unassigned_fields(){
                treat_result_with_memory_error(
                    Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedBus)),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            }
        }
        

        Result::Ok(FoldedValue{bus_slice: Some((symbol.to_string(), bus_slice)), tags: Some(tags_propagated), ..FoldedValue::default()})
    } else if is_tag{
        // in this case we access to the value of a tag (of the complete bus or a field)
        let mut to_do_access = &access_information;
        let mut ref_tag_data = tag_data;
        let mut next_access = access_information.remaining_access.as_ref().unwrap();        
        // we perform all the field accesses, we stop in the previous one to the tag
        while next_access.remaining_access.is_some(){
            let field = to_do_access.field_access.as_ref().unwrap();
            ref_tag_data = ref_tag_data.fields.get(field).unwrap();
            to_do_access = to_do_access.remaining_access.as_ref().unwrap();
            next_access = next_access.remaining_access.as_ref().unwrap();
        }
        // the last access is the tag access
        let tag_access = to_do_access.field_access.as_ref().unwrap();
        let value_tag = ref_tag_data.tags.get(tag_access).unwrap();
        let is_complete = ref_tag_data.remaining_inserts == 0;
        let state = ref_tag_data.definitions.get(tag_access).unwrap();
        if let Some(value_tag) = value_tag { // tag has value
            // access only allowed when (1) it is value defined by user or (2) it is completely assigned
            if state.value_defined || is_complete{
                let a_value = AExpr::Number { value: value_tag.clone() };
                let ae_slice = AExpressionSlice::new(&a_value);
                Result::Ok(FoldedValue { arithmetic_slice: Option::Some(ae_slice), ..FoldedValue::default() })
            } else{
                let error = MemoryError::TagValueNotInitializedAccess;
                treat_result_with_memory_error(
                    Result::Err(error),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            }     
        }
        else {
            let error = MemoryError::TagValueNotInitializedAccess;
            treat_result_with_memory_error(
                Result::Err(error),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        }
    } else{
        // access to a field 

        let resulting_bus = safe_unwrap_to_single(bus_slice, line!());
        let symbol = create_symbol_bus(symbol, &access_information);

        // access to the field
        let field_name = access_information.field_access.as_ref().unwrap();
        let remaining_access = access_information.remaining_access.as_ref().unwrap();


        let result= treat_result_with_memory_error(
            resulting_bus.get_field(field_name, remaining_access),
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,                                                        
        )?;

        // get the tags from the environment
        let mut to_do_access = &access_information;
        let mut ref_tag_data = tag_data;
        // we perform all the field accesses
        while to_do_access.field_access.is_some(){
            let field = to_do_access.field_access.as_ref().unwrap();
            ref_tag_data = ref_tag_data.fields.get(field).unwrap();
            to_do_access = to_do_access.remaining_access.as_ref().unwrap();
        }
        // Compute which tags are propagated 
        let tags_propagated = compute_propagated_tags_bus(&ref_tag_data);

        // match the result and generate the output
        match result{
            FoldedResult::Signal(signals) =>{
                // Generate signal slice and check that all assigned
            
                let result = signal_to_arith(symbol, signals)
                    .map(|s| FoldedValue { 
                        arithmetic_slice: Option::Some(s),
                        tags: Option::Some(tags_propagated),
                        ..FoldedValue::default() 
                    });
                treat_result_with_memory_error(
                    result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )
            },
            FoldedResult::Bus(buses) =>{
                // Check that all the buses are completely assigned

                for i in 0..BusSlice::get_number_of_cells(&buses){
                    let value_left = treat_result_with_memory_error(
                        BusSlice::get_reference_to_single_value_by_index(&buses, i),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
            
                    if value_left.has_unassigned_fields(){
                        treat_result_with_memory_error(
                            Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedBus)),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                    }
                }
                Ok(FoldedValue { 
                    bus_slice: Option::Some((symbol, buses)),
                    tags: Option::Some(tags_propagated),
                    ..FoldedValue::default() 
                })

            },
        }

    }

}

fn execute_component(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<FoldedValue, ()> {
        
    let access_information = treat_accessing_bus(meta, access, program_archive, runtime, flags)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&AExpr::NonQuadratic));
        return Result::Ok(FoldedValue { arithmetic_slice, ..FoldedValue::default() });
    }

    let environment_response =
        ExecutionEnvironment::get_component_res(&runtime.environment, symbol);
    let component_slice = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let memory_response = if runtime.anonymous_components.contains_key(symbol) {
        ComponentSlice::access_values(component_slice, &Vec::new())
    } else{
        ComponentSlice::access_values(component_slice, &access_information.array_access)
    };
    let slice_result = treat_result_with_memory_error(
        memory_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let resulting_component = safe_unwrap_to_single(slice_result, line!());
    
    if let Option::Some(signal_name) = &access_information.field_access {
        let remaining_access = access_information.remaining_access.as_ref().unwrap();
        let symbol = create_symbol_bus(symbol, &access_information);

        if meta.get_type_knowledge().is_tag(){
            // case accessing a tag of a field of the subcomponent
            let result = treat_result_with_memory_error(
                resulting_component.get_tag_value(signal_name, remaining_access),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            let a_value = AExpr::Number { value: result };
            let ae_slice = AExpressionSlice::new(&a_value);
            Result::Ok(FoldedValue { arithmetic_slice: Option::Some(ae_slice), ..FoldedValue::default() })

        } else{
            // case accessing a field
            let (tags, result) = treat_result_with_memory_error(
                resulting_component.get_io_value(signal_name, remaining_access),
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            
            match result{
                FoldedResult::Signal(signals) =>{
                    let result = signal_to_arith(symbol, signals)
                        .map(|s| FoldedValue { 
                            arithmetic_slice: Option::Some(s),
                            tags: Option::Some(tags),
                            ..FoldedValue::default() 
                        });
                    treat_result_with_memory_error(
                        result,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )
                },
                FoldedResult::Bus(buses) =>{
                    // Check that all the buses are completely assigned
    
                    for i in 0..BusSlice::get_number_of_cells(&buses){
                        let value_left = treat_result_with_memory_error(
                            BusSlice::get_reference_to_single_value_by_index(&buses, i),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                
                        if value_left.has_unassigned_fields(){
                            treat_result_with_memory_error(
                                Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedBus)),
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                            )?;
                        }
                    }
                    Ok(FoldedValue { 
                        bus_slice: Option::Some((symbol, buses)),
                        tags: Option::Some(tags),
                        ..FoldedValue::default() 
                    })
    
                }
            }
        }

        

    } else {
            let read_result = if resulting_component.is_ready_initialize() {
                Result::Ok(resulting_component)
            } else {
                Result::Err(MemoryError::InvalidAccess(TypeInvalidAccess::NoInitializedComponent))
            };
    
            let checked_component = treat_result_with_memory_error(
                read_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
    
            Result::Ok(FoldedValue {
                node_pointer: checked_component.node_pointer,
                is_parallel: Some(false),
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
    let templates = program_archive.get_template_names();

    let arg_names = if functions.contains(id) {
        program_archive.get_function_data(id).get_name_of_params()
    } else if templates.contains(id){
        program_archive.get_template_data(id).get_name_of_params()
    } else {
        // case bus
        program_archive.get_bus_data(id).get_name_of_params()
    };

    let mut environment = ExecutionEnvironment::new();
    debug_assert_eq!(arg_names.len(), arg_values.len());
    for (arg_name, arg_value) in arg_names.iter().zip(arg_values) {
        ExecutionEnvironment::add_variable(&mut environment, arg_name, (TagInfo::new(), arg_value.clone()));
    }
    environment
}

fn execute_function_call(
    id: &str,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<(FoldedValue, bool), ()> {
    use std::mem;
    let previous_block = runtime.block_type;
    let previous_conditions = mem::replace(&mut runtime.conditions_state, vec![]);
    runtime.block_type = BlockType::Known;
    let function_body = program_archive.get_function_data(id).get_body_as_vec();
    let (function_result, can_be_simplified) =
        execute_sequence_of_statements(function_body, program_archive, runtime, &mut Option::None, flags, true)?;
    runtime.block_type = previous_block;
    runtime.conditions_state = previous_conditions;
    let return_value = function_result.unwrap();
    debug_assert!(FoldedValue::valid_arithmetic_slice(&return_value));
    Result::Ok((return_value, can_be_simplified))
}



fn execute_template_call(
    id: &str,
    parameter_values: Vec<AExpressionSlice>,
    tag_values: HashMap<String, TagWire>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<FoldedValue, ()> {
    debug_assert!(runtime.block_type == BlockType::Known);
    let is_main = std::mem::replace(&mut runtime.public_inputs, vec![]);
    let is_parallel = program_archive.get_template_data(id).is_parallel();
    let is_custom_gate = program_archive.get_template_data(id).is_custom_gate();
    let args_names = program_archive.get_template_data(id).get_name_of_params();
    let template_body = program_archive.get_template_data(id).get_body_as_vec();
    let mut args_to_values = BTreeMap::new();
    debug_assert_eq!(args_names.len(), parameter_values.len());
    let mut instantiation_name = format!("{}(", id);
    let mut not_empty_name = false;
    for (name, value) in args_names.iter().zip(parameter_values) {
        instantiation_name.push_str(&format!("{},", value.to_string()));
        not_empty_name = true;
        args_to_values.insert(name.clone(), value.clone());
    }
    for (_input, input_tags) in &tag_values{
        // TODO: does not got inside bus
        for (_tag, value) in &input_tags.tags {
            if value.is_none(){
                instantiation_name.push_str("null,");
            }
            else{
                let value = value.clone().unwrap();
                instantiation_name.push_str(&format!("{},", value.to_string()));
            }
            not_empty_name = true;
        }
    }

    if not_empty_name  {
        instantiation_name.pop();
    }
    instantiation_name.push(')');
    let existent_node = runtime.exec_program.identify_node(id, &args_to_values, &tag_values);
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
            tag_values,
            code,
            is_parallel,
            is_custom_gate
        ));
        let (ret, _) = execute_sequence_of_statements(
            template_body,
            program_archive,
            runtime,
            &mut node_wrap,
            flags, 
            true
        )?;
        debug_assert!(ret.is_none());

        let result_check_components = environment_check_all_components_assigned(&runtime.environment);
        match result_check_components{
            Err((error, meta)) =>{
                treat_result_with_memory_error_void(
                    Err(error),
                    &meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            },
            Ok(_) => {},
        }
        let mut new_node = node_wrap.unwrap();


        // we add the tags to the executed template
        // TODO: improve and remove clone
        let outputs = new_node.outputs.clone();
        for output in outputs{
            let to_add = if output.is_bus{
                environment_get_value_tags_bus(&runtime.environment, &output.name)
            } else{
                environment_get_value_tags_signal(&runtime.environment, &output.name)
            };
            for (name, value) in to_add{
                new_node.add_tag_signal(name, value);
            }
        }   
        

        let analysis = std::mem::replace(&mut runtime.analysis, analysis);
        let node_pointer = runtime.exec_program.add_node_to_scheme(new_node, analysis);
        node_pointer
    };
    Result::Ok(FoldedValue { node_pointer: Option::Some(node_pointer), is_parallel: Option::Some(false), ..FoldedValue::default() })
}

fn preexecute_template_call(
    id: &str,
    parameter_values: &[AExpressionSlice],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
) -> Result<FoldedValue, ()> {
    
    pub fn collect_tag_info(
        bus_data: &BusData, 
        program_archive: &ProgramArchive, 
    )-> HashMap<String, TagNames>{
        let mut bus_fields_tags = HashMap::new();
        for (field_name, field_info)  in bus_data.get_fields(){
            let tags = field_info.get_tags();
            
            let fields = match field_info.get_type() {
                WireType::Signal => {
                    None
                },
                WireType::Bus(bus_name) =>{
                    let bus_data = program_archive.get_bus_data(&bus_name);
                    let info = collect_tag_info(bus_data, program_archive);
                    Some(info)
                }
            };
            let tag_name = TagNames{
                tag_names: tags.clone(),
                fields
            };
            bus_fields_tags.insert(field_name.clone(), tag_name);
        }
        bus_fields_tags
        
    }

    debug_assert!(runtime.block_type == BlockType::Known);
    let inputs =  program_archive.get_template_data(id).get_inputs();
    let outputs =  program_archive.get_template_data(id).get_outputs();

    let mut inputs_to_tags = HashMap::new();
    let mut outputs_to_tags = HashMap::new();


    for (name, info_input) in inputs {
        let tags = info_input.get_tags().clone();
        
        let fields = match info_input.get_type() {
            WireType::Signal => {
                None
            },
            WireType::Bus(bus_name) =>{
                let bus_data = program_archive.get_bus_data(&bus_name);
                Some(collect_tag_info(bus_data, program_archive))
            }
        };
        inputs_to_tags.insert(
            name.clone(),
            TagNames{
                tag_names: tags,
                fields
            }
        );
    }

    for (name, info_output) in outputs {
        let tags = info_output.get_tags().clone();
        
        let fields = match info_output.get_type() {
            WireType::Signal => {
                None
            },
            WireType::Bus(bus_name) =>{
                let bus_data = program_archive.get_bus_data(&bus_name);
                Some(collect_tag_info(bus_data, program_archive))
            }
        };
        outputs_to_tags.insert(
            name.clone(),
            TagNames{
                tag_names: tags,
                fields
            }
        );
    }

    let node_wrap = Option::Some(PreExecutedTemplate::new(
        id.to_string(),
        parameter_values.to_vec(),
        inputs_to_tags,
        outputs_to_tags,
    ));

    let new_node = node_wrap.unwrap();
    let node_pointer = runtime.exec_program.add_prenode_to_scheme(new_node);
    Result::Ok(FoldedValue { node_pointer: Option::Some(node_pointer), is_parallel: Option::Some(false), ..FoldedValue::default() })
}

fn execute_bus_call(
    id: &str,
    parameter_values: Vec<AExpressionSlice>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    debug_assert!(runtime.block_type == BlockType::Known);
   
    let args_names = program_archive.get_bus_data(id).get_name_of_params();
    let bus_body = program_archive.get_bus_data(id).get_body_as_vec();
    let mut args_to_values = BTreeMap::new();
    debug_assert_eq!(args_names.len(), parameter_values.len());
    let mut instantiation_name = format!("{}(", id);
    let mut not_empty_name = false;
    
    for (name, value) in args_names.iter().zip(parameter_values) {
        instantiation_name.push_str(&format!("{},", value.to_string()));
        not_empty_name = true;
        args_to_values.insert(name.clone(), value.clone());
    }

    if not_empty_name  {
        instantiation_name.pop();
    }
    instantiation_name.push(')');

    let existent_node = runtime.exec_program.identify_bus_node(id, &args_to_values);
    let node_pointer = if let Option::Some(pointer) = existent_node {
        pointer
    } else {
        let analysis =
            std::mem::replace(&mut runtime.analysis, Analysis::new(program_archive.id_max));
        let mut node = ExecutedBus::new(
            id.to_string(),
            instantiation_name,
            args_to_values,
        );
        execute_sequence_of_bus_statements(
            bus_body,
            program_archive,
            runtime,
            &mut node,
            flags, 
        )?;

        let analysis = std::mem::replace(&mut runtime.analysis, analysis);
        let node_pointer = runtime.exec_program.add_bus_node_to_scheme(node, analysis);
        node_pointer
    };
    Result::Ok(FoldedValue { bus_node_pointer: Option::Some(node_pointer), ..FoldedValue::default() })
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
        Complement => AExpr::complement(value, field),
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
    flags: FlagsExecution
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
                let index_fold = execute_expression(index, program_archive, runtime, flags)?;
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
        match AExpr::get_usize(ae_index) {
            Some(index) => { sc_indexes.push(index); },
            None => { return Option::None; },
        }
    }
    Option::Some(sc_indexes)
}

fn cast_index(ae_index: &AExpr) -> Option<SliceCapacity> {
    if !ae_index.is_number() {
        return Option::None;
    }
    match AExpr::get_usize(ae_index) {
        Option::Some(index) => { Option::Some(index) },
        Option::None => {  Option::None },
    }
}

fn treat_accessing(
    meta: &Meta,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<AccessingInformation, ()> {
    let (ae_before_signal, signal_name, signal_index) =
        treat_indexing(0, access, program_archive, runtime, flags)?;
    let (ae_after_signal, tag_name , _tag_index) =
        treat_indexing(signal_index + 1, access, program_archive, runtime, flags)?;
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
    let tag_access = tag_name;
    let (before_signal, after_signal) = if !undefined {
        (possible_before_indexing.unwrap(), possible_after_indexing.unwrap())
    } else {
        (Vec::new(), Vec::new())
    };
    Result::Ok(AccessingInformation { undefined, before_signal, after_signal, signal_access, tag_access})
}


fn treat_accessing_bus(
    meta: &Meta,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution
) -> Result<AccessingInformationBus, ()> {

    fn treat_accessing_bus_index(
        index: usize,
        meta: &Meta,
        access: &[Access],
        program_archive: &ProgramArchive,
        runtime: &mut RuntimeInformation,
        flags: FlagsExecution
    ) -> Result<AccessingInformationBus, ()>{
        
        let (ae_before_signal, field_access, signal_index) =
            treat_indexing(index, access, program_archive, runtime, flags)?;
        
        treat_result_with_memory_error(
            valid_indexing(&ae_before_signal),
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;

        let mut remaining_access = if signal_index < access.len(){
            Some(Box::new(
                treat_accessing_bus_index(signal_index + 1, meta, access, program_archive, runtime, flags)?)
            )
        } else{
            None
        };

        let possible_before_indexing = cast_indexing(&ae_before_signal);

        let remaining_access_undefined = remaining_access.is_some() && remaining_access.as_ref().unwrap().undefined;

        let undefined = possible_before_indexing.is_none() || remaining_access_undefined;

        let array_access = if undefined {
            Vec::new()
        } else {
            possible_before_indexing.unwrap()
        };
        if undefined{
            remaining_access = None
        };

    Result::Ok(AccessingInformationBus { undefined, array_access, remaining_access, field_access})

    }

    treat_accessing_bus_index(
        0,
        meta,
        access,
        program_archive,
        runtime,
        flags
    )

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
fn safe_unwrap_to_valid_node_pointer(folded_value: FoldedValue, line: u32) -> (NodePointer, bool) {
    debug_assert!(FoldedValue::valid_node_pointer(&folded_value), "Caused by call at {}", line);
    (folded_value.node_pointer.unwrap(), folded_value.is_parallel.unwrap())
}
fn safe_unwrap_to_valid_bus_node_pointer(folded_value: FoldedValue, line: u32) -> NodePointer {
    debug_assert!(FoldedValue::valid_bus_node_pointer(&folded_value), "Caused by call at {}", line);
    folded_value.bus_node_pointer.unwrap()
}
fn safe_unwrap_to_bus_slice(folded_value: FoldedValue, line: u32) -> (String, BusSlice) {
    debug_assert!(FoldedValue::valid_arithmetic_slice(&folded_value), "Caused by call at {}", line);
    folded_value.bus_slice.unwrap()
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

fn treat_result_with_memory_error_void(
    memory_error: Result<(), MemoryError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<(), ()> {
    use ReportCode::RuntimeError;
    match memory_error {
        Result::Ok(()) => Result::Ok(()),
        Result::Err(MemoryError::MismatchedDimensionsWeak(dim_given, dim_original)) => {
                    
                    let report = if dim_given <  dim_original{
                        Report::warning(
                            format!("Typing warning: Mismatched dimensions, assigning to an array an expression of smaller length, the remaining positions are not modified. Initially all variables are initialized to 0.\n  Expected length: {}, given {}",
                                dim_original, dim_given),
                            RuntimeError
                        )
                    } else{
                        Report::warning(
                            format!("Typing warning: Mismatched dimensions, assigning to an array an expression of greater length, the remaining positions of the expression are not assigned to the array.\n  Expected length: {}, given {}",
                                dim_original, dim_given),
                            RuntimeError
                        )
                    };
                    add_report_to_runtime(report, meta, runtime_errors, call_trace);
                    Ok(())
                },
        Result::Err(memory_error) => {
            let report = match memory_error {
                MemoryError::InvalidAccess(type_invalid_access) => {
                    match type_invalid_access{
                        TypeInvalidAccess::MissingInputs(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to an output signal of a component with not all its inputs initialized.\n Missing input: {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::MissingInputTags(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to a signal of a component with not all its inputs with tags initialized.\n Missing input (with tags): {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid access: trying to access to a component that is not initialized" .to_string(),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedSignal =>{
                            Report::error("Exception caused by invalid access: trying to access to a signal that is not initialized" .to_string(),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedBus =>{
                            Report::error("Exception caused by invalid access: trying to access to a bus whose fields have not been completely initialized" .to_string(),
                                RuntimeError)
                        }
                    }
                }
                MemoryError::AssignmentError(type_asig_error) => {
                    match type_asig_error{
                         TypeAssignmentError::MultipleAssignmentsComponent =>{
                            Report::error(
                                format!("Exception caused by invalid assignment\n The component has been assigned previously"),
                                RuntimeError)
                        },
                        TypeAssignmentError::MultipleAssignmentsBus =>{
                            Report::error(
                                format!("Exception caused by invalid assignment\n Bus contains fields that have been previously initialized"),
                                RuntimeError)
                        },
                        TypeAssignmentError::MultipleAssignments(meta) =>{
                            let mut rep = Report::error(
                                format!("Exception caused by invalid assignment\n Signal has been already assigned"),
                                RuntimeError);
                            rep.add_secondary(meta.file_location(), meta.get_file_id(), Option::Some("This is the previous assignment to the variable".to_string()));
                            rep
                        },
                        TypeAssignmentError::AssignmentInput(signal) => Report::error(
                            format!("Invalid assignment: input signals of a template already have a value when the template is executed and cannot be re-assigned. \n Problematic input signal: {}",
                                signal),
                            RuntimeError,
                        ),
                        TypeAssignmentError::AssignmentOutput =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to an output signal of a component".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to a signal of a component that has not been initialized".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::DifferentBusInstances =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a different instance of the bus. The instances of the buses should be equal".to_string(),
                                RuntimeError)
                        }
                    }
                },
                MemoryError::OutOfBoundsError => {
                    Report::error("Out of bounds exception".to_string(), RuntimeError)
                },
                MemoryError::MismatchedDimensions(given, orig) => {
                    Report::error(
                        format!("Typing error found: mismatched dimensions.\n Expected length: {}, given {}",
                            orig, given),
                         RuntimeError)
                },
                MemoryError::MismatchedInstances => {
                    Report::error(
                        format!("Typing error found: mismatched instances.\n Trying to compare two different instances of a bus, the instances must be equal"),
                         RuntimeError)
                },

                MemoryError::UnknownSizeDimension => {
                    Report::error("Array dimension with unknown size".to_string(), RuntimeError)
                },
                MemoryError::AssignmentMissingTags(signal, tag) => Report::error(
                    format!("Invalid assignment: missing tags required by input signal. \n Missing tag: input signal {} requires tag {}",
                            signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagAfterInit => Report::error(
                    "Invalid assignment: tags cannot be assigned to a signal already initialized".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagTwice => Report::error(
                    "Invalid assignment: this tag already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInputTwice(signal, tag) => Report::error(
                    format!("Invalid assignment: tags required by the input signal always have to have the same value. \n Problematic tag: input signal {} already has a different value for tag {}",
                        signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInput => Report::error(
                    "Invalid assignment: this tag belongs to an input which already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::MismatchedDimensionsWeak(..) => unreachable!()
                ,
                MemoryError::TagValueNotInitializedAccess => Report::error(
                    "Tag value has not been previously initialized".to_string(), 
                    RuntimeError)
                , 
                MemoryError::MissingInputs(name) => Report::error(
                    format!("Component {} is created but not all its inputs are initialized", name),
                    RuntimeError,
                )
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

pub fn treat_result_with_memory_error<C>(
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
                MemoryError::InvalidAccess(type_invalid_access) => {
                    match type_invalid_access{
                        TypeInvalidAccess::MissingInputs(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to an output signal of a component with not all its inputs initialized.\n Missing input: {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::MissingInputTags(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to a signal of a component with not all its inputs with tags initialized.\n Missing input (with tags): {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid access: trying to access to a component that is not initialized" .to_string(),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedSignal =>{
                            Report::error("Exception caused by invalid access: trying to access to a signal that is not initialized" .to_string(),
                                RuntimeError)
                        }
                        TypeInvalidAccess::NoInitializedBus =>{
                            Report::error("Exception caused by invalid access: trying to access to a bus whose fields have not been completely initialized" .to_string(),
                                RuntimeError)
                        }
                    }
                },
                MemoryError::AssignmentError(type_asig_error) => {
                    match type_asig_error{
                        TypeAssignmentError::MultipleAssignmentsComponent =>{
                            Report::error(
                                format!("Exception caused by invalid assignment\n The component has been assigned previously"),
                                RuntimeError)
                        },
                        TypeAssignmentError::MultipleAssignmentsBus =>{
                            Report::error(
                                format!("Exception caused by invalid assignment\n Bus contains fields that have been previously initialized"),
                                RuntimeError)
                        },
                        TypeAssignmentError::MultipleAssignments(meta) =>{
                            let mut rep = Report::error(
                                format!("Exception caused by invalid assignment\n Signal has been already assigned"),
                                RuntimeError);
                            rep.add_secondary(meta.file_location(), meta.get_file_id(), Option::Some("This is the previous assignment to the variable".to_string()));
                            rep
                        },
                        TypeAssignmentError::AssignmentInput(signal) => Report::error(
                            format!("Invalid assignment: input signals of a template already have a value when the template is executed and cannot be re-assigned. \n Problematic input signal: {}",
                                signal),
                            RuntimeError,
                        ),
                        TypeAssignmentError::AssignmentOutput =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to an output signal of a component".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to a signal of a component that has not been initialized".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::DifferentBusInstances =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a different instance of the bus. The instances of the buses should be equal".to_string(),
                                RuntimeError)
                        }
                    }
                },
                MemoryError::AssignmentMissingTags(signal, tag) => Report::error(
                    format!("Invalid assignment: missing tags required by input signal. \n Missing tag: input signal {} requires tag {}",
                            signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagAfterInit => Report::error(
                    "Invalid assignment: tags cannot be assigned to a signal already initialized".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagTwice => Report::error(
                    "Invalid assignment: this tag already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInputTwice(signal, tag) => Report::error(
                    format!("Invalid assignment: tags required by the input signal always have to have the same value. \n Problematic tag: input signal {} already has a different value for tag {}",
                        signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInput => Report::error(
                    "Invalid assignment: this tag belongs to an input which already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::OutOfBoundsError => {
                    Report::error("Out of bounds exception".to_string(), RuntimeError)
                },
                MemoryError::MismatchedDimensions(given, orig) => {
                    Report::error(
                        format!("Typing error found: mismatched dimensions.\n Expected length: {}, given {}",
                            orig, given),
                         RuntimeError)
                },
                MemoryError::MismatchedInstances => {
                    Report::error(
                        format!("Typing error found: mismatched instances.\n Trying to compare two different instances of a bus, the instances must be equal"),
                         RuntimeError)
                },
                MemoryError::UnknownSizeDimension => {
                    Report::error("Array dimension with unknown size".to_string(), RuntimeError)
                }
                MemoryError::TagValueNotInitializedAccess => {
                    Report::error("Tag value has not been previously initialized".to_string(), RuntimeError)

                } 
                MemoryError::MismatchedDimensionsWeak(..) => {
                    unreachable!()
                },
                MemoryError::MissingInputs(name) => Report::error(
                    format!("Component {} is created but not all its inputs are initialized", name),
                    RuntimeError,
                )
                
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
                UnknownTemplate => Report::error(
                    "Every component instantiation must be resolved during the constraint generation phase. This component declaration uses a value that can be unknown during the constraint generation phase.".to_string(),
                    ReportCode::RuntimeError,
                ),
                NonValidTagAssignment => Report::error(
                    "Tags cannot be assigned to values that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                ),
                FalseAssert => {
                    Report::error("False assert reached".to_string(), ReportCode::RuntimeError)
                }
                ArraySizeTooBig => Report::error(
                    "The size of the array is expected to be a usize".to_string(),
                    ReportCode::RuntimeError,
                ),
                ConstraintInUnknown => Report::error(
                    "There are constraints depending on the value of a condition that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                ),
                DeclarationInUnknown => Report::error(
                    "There are signal or component declarations depending on the value of a condition that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                ),
                TagAssignmentInUnknown => Report::error(
                    "There are tag assignments depending on the value of a condition that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                )
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_execution_warning<C>(
    execution_error: Result<C, ExecutionWarning>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<(), ()> {
    use ExecutionWarning::*;
    match execution_error {
        Result::Ok(_) => Result::Ok(()),
        Result::Err(execution_error) => {
            let report = match execution_error {
                CanBeQuadraticConstraintSingle() => {
                    let msg = format!(
                        "Consider using <== instead of <-- to add the corresponding constraint.\n The constraint representing the assignment satisfies the R1CS format and can be added to the constraint system."
                    );
                    Report::warning(
                        msg,
                        ReportCode::RuntimeWarning,
                    )
                },  
                CanBeQuadraticConstraintMultiple(positions) =>{
                    let mut msg_positions = positions[0].clone();
                    for i in 1..positions.len(){
                        msg_positions = format!("{}, {}", msg_positions, positions[i].clone()) 
                    };

                    let msg = format!(
                        "Consider using <== instead of <-- for some of positions of the array of signals being assigned.\n The constraints representing the assignment of the positions {} satisfy the R1CS format and can be added to the constraint system.",
                        msg_positions
                    );
                    Report::warning(
                        msg,
                        ReportCode::RuntimeWarning,
                    )
                }


            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Ok(())
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



