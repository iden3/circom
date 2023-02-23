use crate::environment_utils::environment::ExecutionEnvironment as EE;
use crate::environment_utils::slice_types::{TagInfo, AExpressionSlice};
use circom_algebra::algebra::ArithmeticExpression;
use compiler::hir::very_concrete_program::{Argument, TemplateInstance};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use program_structure::ast::{Expression, Meta, Statement};
use program_structure::error_definition::ReportCollection;
use program_structure::program_archive::ProgramArchive;
use std::collections::HashMap;

type CCResult = Result<(), ReportCollection>;

struct Context<'a> {
    inside_template: bool,
    environment: &'a EE,
    program_archive: &'a ProgramArchive,
}

pub fn manage_functions(program_archive: &mut ProgramArchive, flag_verbose: bool, prime: &String) -> CCResult {
    let mut reports = vec![];
    let mut processed = HashMap::new();
    for (name, data) in program_archive.get_functions() {
        let mut code = data.get_body().clone();
        let environment = EE::new();
        let context =
            Context { program_archive, inside_template: false, environment: &environment };
        treat_statement(&mut code, &context, &mut reports, flag_verbose, prime);
        processed.insert(name.clone(), code);
    }
    for (k, v) in processed {
        program_archive.get_mut_function_data(&k).replace_body(v);
    }
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

pub fn compute_vct(
    instances: &mut Vec<TemplateInstance>,
    program_archive: &ProgramArchive,
    flag_verbose: bool,
    prime: &String
) -> CCResult {
    let mut reports = vec![];
    for instance in instances {
        let environment = transform_header_into_environment(&instance.header);
        let context = Context { program_archive, inside_template: true, environment: &environment };
        treat_statement(&mut instance.code, &context, &mut reports, flag_verbose, prime);
    }
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

fn transform_header_into_environment(header: &[Argument]) -> EE {
    let mut execution_environment = EE::new();
    for arg in header {
        let name = arg.name.clone();
        let slice = argument_into_slice(arg);
        execution_environment.add_variable(&name, (TagInfo::new(), slice));
    }
    execution_environment
}

fn argument_into_slice(argument: &Argument) -> AExpressionSlice {
    use ArithmeticExpression::Number;
    let arithmetic_expressions: Vec<ArithmeticExpression<String>> =
        argument.values.iter().map(|v| Number { value: v.clone() }).collect();
    let dimensions = argument.lengths.clone();
    AExpressionSlice::new_array(dimensions, arithmetic_expressions)
}

fn treat_statement(stmt: &mut Statement, context: &Context, reports: &mut ReportCollection, flag_verbose: bool, prime: &String) {
    if stmt.is_initialization_block() {
        treat_init_block(stmt, context, reports, flag_verbose, prime)
    } else if stmt.is_block() {
        treat_block(stmt, context, reports, flag_verbose, prime)
    } else if stmt.is_if_then_else() {
        treat_conditional(stmt, context, reports, flag_verbose, prime)
    } else if stmt.is_while() {
        treat_while(stmt, context, reports, flag_verbose, prime)
    } else if stmt.is_declaration(){
        treat_declaration(stmt, context, reports, flag_verbose, prime)
    } else {
    }
}

fn treat_init_block(stmt: &mut Statement, context: &Context, reports: &mut ReportCollection, flag_verbose: bool, prime: &String) {
    use Statement::InitializationBlock;
    if let InitializationBlock { initializations, .. } = stmt {
        for init in initializations {
            if init.is_declaration() {
                treat_declaration(init, context, reports, flag_verbose, prime)
            }
        }
    } else {
        unreachable!()
    }
}

fn treat_block(stmt: &mut Statement, context: &Context, reports: &mut ReportCollection, flag_verbose: bool, prime: &String) {
    use Statement::Block;
    if let Block { stmts, .. } = stmt {
        for s in stmts {
            treat_statement(s, context, reports, flag_verbose, prime);
        }
    } else {
        unreachable!()
    }
}

fn treat_while(stmt: &mut Statement, context: &Context, reports: &mut ReportCollection, flag_verbose: bool, prime: &String) {
    use Statement::While;
    if let While { stmt, .. } = stmt {
        treat_statement(stmt, context, reports, flag_verbose, prime);
    } else {
        unreachable!()
    }
}

fn treat_conditional(stmt: &mut Statement, context: &Context, reports: &mut ReportCollection, flag_verbose: bool, prime: &String) {
    use Statement::IfThenElse;
    if let IfThenElse { if_case, else_case, .. } = stmt {
        treat_statement(if_case, context, reports, flag_verbose, prime);
        if let Option::Some(s) = else_case {
            treat_statement(s, context, reports, flag_verbose, prime);
        }
    } else {
        unreachable!()
    }
}

fn treat_declaration(stmt: &mut Statement, context: &Context, reports: &mut ReportCollection, flag_verbose: bool, prime: &String) {
    use Statement::Declaration;
    use program_structure::ast::VariableType::AnonymousComponent;
    if let Declaration { meta, dimensions, xtype, .. } = stmt {
        let mut concrete_dimensions = vec![];
        match  xtype {
            AnonymousComponent => {
                meta.get_mut_memory_knowledge().set_concrete_dimensions(vec![]);
            },
            _ => {
                for d in dimensions.iter_mut() {
                    let execution_response = treat_dimension(d, context, reports, flag_verbose, prime);
                    if let Option::Some(v) = execution_response {
                        concrete_dimensions.push(v);
                    } else {
                        report_invalid_dimension(meta, reports);
                    }
                }
                meta.get_mut_memory_knowledge().set_concrete_dimensions(concrete_dimensions);
            }
        }
    } else {
        unreachable!()
    }
}

fn treat_dimension(
    dim: &Expression,
    context: &Context,
    reports: &mut ReportCollection,
    flag_verbose: bool, 
    prime: &String,
) -> Option<usize> {
    use crate::execute::execute_constant_expression;
    if context.inside_template && !dim.is_number() {
        Option::None
    } else if let Expression::Number(_, v) = dim {
        transform_big_int_to_usize(v)
    } else {
        let program = context.program_archive;
        let env = context.environment;
        let execution_result = execute_constant_expression(dim, program, env.clone(), flag_verbose, prime);
        match execution_result {
            Result::Err(mut r) => {
                reports.append(&mut r);
                Option::None
            }
            Result::Ok(v) => transform_big_int_to_usize(&v),
        }
    }
}

fn transform_big_int_to_usize(v: &BigInt) -> Option<usize> {
    v.to_usize()
}

fn report_invalid_dimension(meta: &Meta, reports: &mut ReportCollection) {
    use program_structure::error_code::ReportCode;
    use program_structure::error_definition::Report;
    let error_code = ReportCode::InvalidArraySize;
    let msg = "Invalid array size".to_string();
    let mut report = Report::error(msg, error_code);
    let message = "This expression can not be used as an array size".to_string();
    report.add_primary(meta.file_location(), meta.get_file_id(), message);
    reports.push(report);
}
