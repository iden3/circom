use program_structure::ast::{Access, Expression, Meta, Statement, LogArgument};
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::{self, FileID, FileLocation};
use program_structure::function_data::FunctionInfo;
use program_structure::program_archive::ProgramArchive;
use program_structure::template_data::TemplateInfo;
use std::collections::HashSet;
type Block = HashSet<String>;
type Environment = Vec<Block>;

pub fn check_naming_correctness(program_archive: &ProgramArchive) -> Result<(), ReportCollection> {
    let template_info = program_archive.get_templates();
    let function_info = program_archive.get_functions();
    let mut reports = ReportCollection::new();
    let mut instances = Vec::new();
    for (_, data) in template_info {
        let instance = (
            data.get_file_id(),
            data.get_param_location(),
            data.get_name_of_params(),
            data.get_body_as_vec(),
        );
        instances.push(instance);
    }
    for (_, data) in function_info {
        let instance = (
            data.get_file_id(),
            data.get_param_location(),
            data.get_name_of_params(),
            data.get_body_as_vec(),
        );
        instances.push(instance);
    }
    if let Err(mut r) = analyze_main(program_archive) {
        reports.append(&mut r);
    }
    for (file_id, param_location, params_names, body) in instances {
        let res = analyze_symbols(
            file_id,
            param_location,
            params_names,
            body,
            function_info,
            template_info,
        );
        if let Result::Err(mut r) = res {
            reports.append(&mut r);
        }
    }
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

fn analyze_main(program: &ProgramArchive) -> Result<(), Vec<Report>> {
    let call = program.get_main_expression();
    let signals = program.get_public_inputs_main_component();
    let template_info = program.get_templates();
    let function_info = program.get_functions();

    let mut reports = vec![];
    if let Expression::Call { id, .. } = call {
        if program.contains_template(id) {
            let inputs = program.get_template_data(id).get_inputs();
            for signal in signals {
                if !inputs.contains_key(signal) {
                    let mut report = Report::error(
                        format!("Invalid public list"),
                        ReportCode::SameSymbolDeclaredTwice,
                    );
                    report.add_primary(
                        call.get_meta().location.clone(),
                        call.get_meta().get_file_id(),
                        format!("{} is not an input signal", signal),
                    );
                    reports.push(report);
                }
            }
        }
    }
    let environment = Environment::new();
    analyze_expression(
        call,
        call.get_meta().get_file_id(),
        function_info,
        template_info,
        &mut reports,
        &environment,
    );

    if reports.is_empty() { Ok(()) } else { Err(reports) }
}

pub fn analyze_symbols(
    file_id: FileID,
    param_location: FileLocation,
    params_names: &[String],
    body: &[Statement],
    function_info: &FunctionInfo,
    template_info: &TemplateInfo,
) -> Result<(), ReportCollection> {
    let mut param_name_collision = false;
    let mut reports = ReportCollection::new();
    let mut environment = Environment::new();
    environment.push(Block::new());
    for param in params_names.iter() {
        let success = add_symbol_to_block(&mut environment, param);
        param_name_collision = param_name_collision || !success;
    }
    if param_name_collision {
        let mut report =
            Report::error(format!("Symbol declared twice"), ReportCode::SameSymbolDeclaredTwice);
        report.add_primary(
            param_location.clone(),
            file_id.clone(),
            format!("Declaring same symbol twice"),
        );
        reports.push(report);
    }
    for stmt in body.iter() {
        analyze_statement(
            stmt,
            file_id,
            function_info,
            template_info,
            &mut reports,
            &mut environment,
        );
    }
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

fn symbol_in_environment(environment: &Environment, symbol: &String) -> bool {
    for block in environment.iter() {
        if block.contains(symbol) {
            return true;
        }
    }
    false
}

fn add_symbol_to_block(environment: &mut Environment, symbol: &String) -> bool {
    let last_block = environment.last_mut().unwrap();
    if last_block.contains(symbol) {
        return false;
    }
    last_block.insert(symbol.clone());
    true
}

fn analyze_statement(
    stmt: &Statement,
    file_id: FileID,
    function_info: &FunctionInfo,
    template_info: &TemplateInfo,
    reports: &mut ReportCollection,
    environment: &mut Environment,
) {
    match stmt {
        Statement::MultSubstitution { .. } => unreachable!(),
        Statement::Return { value, .. } => {
            analyze_expression(value, file_id, function_info, template_info, reports, environment)
        }
        Statement::UnderscoreSubstitution { rhe, .. } => {
            analyze_expression(rhe, file_id, function_info, template_info, reports, environment);
        }
        Statement::Substitution { meta, var, access, rhe, .. } => {
            analyze_expression(rhe, file_id, function_info, template_info, reports, environment);
            treat_variable(
                meta,
                var,
                access,
                file_id,
                function_info,
                template_info,
                reports,
                environment,
            );
        }
        Statement::ConstraintEquality { lhe, rhe, .. } => {
            analyze_expression(lhe, file_id, function_info, template_info, reports, environment);
            analyze_expression(rhe, file_id, function_info, template_info, reports, environment);
        }
        Statement::InitializationBlock { initializations, .. } => {
            for initialization in initializations.iter() {
                analyze_statement(
                    initialization,
                    file_id,
                    function_info,
                    template_info,
                    reports,
                    environment,
                );
            }
        }
        Statement::Declaration { meta, name, dimensions, .. } => {
            for index in dimensions {
                analyze_expression(
                    index,
                    file_id,
                    function_info,
                    template_info,
                    reports,
                    environment,
                );
            }
            if !add_symbol_to_block(environment, name) {
                let mut report = Report::error(
                    format!("Symbol declared twice"),
                    ReportCode::SameSymbolDeclaredTwice,
                );
                report.add_primary(
                    meta.location.clone(),
                    file_id.clone(),
                    format!("Declaring same symbol twice"),
                );
                reports.push(report);
            }
        }
        Statement::LogCall { args, .. } => {
            for logarg in args {
                if let LogArgument::LogExp(arg) = logarg {
                    analyze_expression(arg, file_id, function_info, template_info, reports, environment);
                }
            }
        }
        Statement::Assert { arg, .. } => {
            analyze_expression(arg, file_id, function_info, template_info, reports, environment)
        }
        Statement::Block { stmts, .. } => {
            environment.push(Block::new());
            for block_stmt in stmts.iter() {
                analyze_statement(
                    block_stmt,
                    file_id,
                    function_info,
                    template_info,
                    reports,
                    environment,
                );
            }
            environment.pop();
        }
        Statement::While { stmt, cond, .. } => {
            analyze_expression(cond, file_id, function_info, template_info, reports, environment);
            analyze_statement(stmt, file_id, function_info, template_info, reports, environment);
        }
        Statement::IfThenElse { cond, if_case, else_case, .. } => {
            analyze_expression(cond, file_id, function_info, template_info, reports, environment);
            analyze_statement(if_case, file_id, function_info, template_info, reports, environment);
            if let Option::Some(else_stmt) = else_case {
                analyze_statement(
                    else_stmt,
                    file_id,
                    function_info,
                    template_info,
                    reports,
                    environment,
                );
            }
        }
    }
}

fn treat_variable(
    meta: &Meta,
    name: &String,
    access: &Vec<Access>,
    file_id: FileID,
    function_info: &FunctionInfo,
    template_info: &TemplateInfo,
    reports: &mut ReportCollection,
    environment: &Environment,
) {
    if !symbol_in_environment(environment, name) {
        let mut report = Report::error(format!("Undeclared symbol"), ReportCode::NonExistentSymbol);
        report.add_primary(
            file_definition::generate_file_location(meta.get_start(), meta.get_end()),
            file_id.clone(),
            format!("Using unknown symbol"),
        );
        reports.push(report);
    }
    for acc in access.iter() {
        if let Access::ArrayAccess(index) = acc {
            analyze_expression(index, file_id, function_info, template_info, reports, environment);
        }
    }
}

fn analyze_expression(
    expression: &Expression,
    file_id: FileID,
    function_info: &FunctionInfo,
    template_info: &TemplateInfo,
    reports: &mut ReportCollection,
    environment: &Environment,
) {
    match expression {
        Expression::InfixOp { lhe, rhe, .. } => {
            analyze_expression(lhe, file_id, function_info, template_info, reports, environment);
            analyze_expression(rhe, file_id, function_info, template_info, reports, environment);
        }
        Expression::PrefixOp { rhe, .. } => {
            analyze_expression(rhe, file_id, function_info, template_info, reports, environment)
        }
        Expression::ParallelOp { rhe, .. } => {
            analyze_expression(rhe, file_id, function_info, template_info, reports, environment)
        }
        Expression::InlineSwitchOp { cond, if_true, if_false, .. } => {
            analyze_expression(cond, file_id, function_info, template_info, reports, environment);
            analyze_expression(
                if_true,
                file_id,
                function_info,
                template_info,
                reports,
                environment,
            );
            analyze_expression(
                if_false,
                file_id,
                function_info,
                template_info,
                reports,
                environment,
            );
        }
        Expression::Variable { meta, name, access, .. } => treat_variable(
            meta,
            name,
            access,
            file_id,
            function_info,
            template_info,
            reports,
            environment,
        ),
        Expression::Call { meta, id, args, .. } => {
            if !function_info.contains_key(id) && !template_info.contains_key(id) {
                let mut report =
                    Report::error(format!("Calling symbol"), ReportCode::NonExistentSymbol);
                report.add_primary(
                    file_definition::generate_file_location(meta.get_start(), meta.get_end()),
                    file_id.clone(),
                    format!("Calling unknown symbol"),
                );
                reports.push(report);
                return;
            }
            let expected_num_of_params = if function_info.contains_key(id) {
                function_info.get(id).unwrap().get_num_of_params()
            } else {
                template_info.get(id).unwrap().get_num_of_params()
            };
            if args.len() != expected_num_of_params {
                let mut report = Report::error(
                    format!("Calling function with wrong number of arguments"),
                    ReportCode::FunctionWrongNumberOfArguments,
                );
                report.add_primary(
                    file_definition::generate_file_location(meta.get_start(), meta.get_end()),
                    file_id.clone(),
                    format!("Got {} params, {} where expected", args.len(), expected_num_of_params),
                );
                reports.push(report);
                return;
            }
            for arg in args.iter() {
                analyze_expression(
                    arg,
                    file_id,
                    function_info,
                    template_info,
                    reports,
                    environment,
                );
            }
        }
        Expression::ArrayInLine { values, .. } => {
            for value in values.iter() {
                analyze_expression(
                    value,
                    file_id,
                    function_info,
                    template_info,
                    reports,
                    environment,
                );
            }
        }
        Expression::UniformArray{ value, dimension, .. } => {
            analyze_expression(
                value,
                file_id,
                function_info,
                template_info,
                reports,
                environment,
            );
            analyze_expression(
                dimension,
                file_id,
                function_info,
                template_info,
                reports,
                environment,
            );
        },
        _ => {}
    }
}
