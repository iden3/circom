use super::analyzers::*;
use super::decorators::*;
use program_structure::error_definition::ReportCollection;
use program_structure::program_archive::ProgramArchive;

pub fn check_types(
    program_archive: &mut ProgramArchive,
) -> Result<ReportCollection, ReportCollection> {
    let mut errors = ReportCollection::new();
    let mut warnings = ReportCollection::new();

    // Structural analyses
    program_level_analyses(program_archive, &mut errors);
    if !errors.is_empty() {
        return Result::Err(errors);
    }

    template_level_analyses(program_archive, &mut errors);
    if !errors.is_empty() {
        return Result::Err(errors);
    }

    function_level_analyses(program_archive, &mut errors);
    if !errors.is_empty() {
        return Result::Err(errors);
    }

    bus_level_analyses(program_archive, &mut errors);
    if !errors.is_empty() {
        return Result::Err(errors);
    }

    // Decorators
    template_level_decorators(program_archive, &mut errors);
    if !errors.is_empty() {
        return Result::Err(errors);
    }

    function_level_decorators(program_archive, &mut errors);
    if !errors.is_empty() {
        return Result::Err(errors);
    }

    bus_level_decorators(program_archive, &mut errors);
    if !errors.is_empty() {
        return Result::Err(errors);
    }

    // Type analysis
    let typing_result = type_check(program_archive);
    match typing_result {
        Err(mut type_reports) => {
            errors.append(&mut type_reports);
            return Result::Err(errors);
        }
        Ok(info) => {
            for name in program_archive.get_function_names().clone() {
                if !info.reached.contains(&name) {
                    program_archive.remove_function(&name)
                }
            }
            for name in program_archive.get_template_names().clone() {
                if !info.reached.contains(&name) {
                    program_archive.remove_template(&name)
                }
            }
            for name in program_archive.get_bus_names().clone() {
                if !info.reached.contains(&name) {
                    program_archive.remove_bus(&name)
                }
            }
        }
    }

    // Semantics analyses
    semantic_analyses(program_archive, &mut errors, &mut warnings);

    if !errors.is_empty() {
        Result::Err(errors)
    } else {
        Result::Ok(warnings)
    }
}

fn program_level_analyses(program_archive: &ProgramArchive, reports: &mut ReportCollection) {
    let symbols_in_body_well_defined_result = check_naming_correctness(program_archive);
    if let Result::Err(mut symbols_in_body_well_defined_reports) =
        symbols_in_body_well_defined_result
    {
        reports.append(&mut symbols_in_body_well_defined_reports);
    }
}

fn template_level_analyses(program_archive: &ProgramArchive, reports: &mut ReportCollection) {
    for template_data in program_archive.get_templates().values() {
        let no_returns_in_template_result = free_of_returns(template_data);
        let signal_declaration_result = check_signal_correctness(template_data);
        if let Result::Err(mut no_returns_reports) = no_returns_in_template_result {
            reports.append(&mut no_returns_reports);
        }
        if let Result::Err(mut signal_declaration_reports) = signal_declaration_result {
            reports.append(&mut signal_declaration_reports);
        }
    }
}

fn template_level_decorators(
    program_archive: &mut ProgramArchive,
    reports: &mut ReportCollection,
) {
    component_type_inference::inference(program_archive);
    let program_archive2 = program_archive.clone();
    for template_data in program_archive.get_mut_templates().values_mut() {
        reports.append(&mut type_reduction::reduce_template(template_data,&program_archive2));
    }
}

fn function_level_analyses(program_archive: &ProgramArchive, reports: &mut ReportCollection) {
    let function_names = program_archive.get_function_names();
    for function_data in program_archive.get_functions().values() {
        let free_of_template_elements_result = free_of_template_elements(function_data, function_names);
        let return_check_result = all_paths_with_return_check(function_data);
        if let Result::Err(mut functions_free_of_template_elements_reports) = free_of_template_elements_result {
            reports.append(&mut functions_free_of_template_elements_reports);
        }
        if let Result::Err(functions_all_paths_with_return_statement_report) = return_check_result {
            reports.push(functions_all_paths_with_return_statement_report);
        }
    }
}

fn function_level_decorators(program_archive: &mut ProgramArchive, reports: &mut ReportCollection) {
    let program_archive2 = program_archive.clone();
    for function_data in program_archive.get_mut_functions().values_mut() {
        let mut constant_handler_reports =
            constants_handler::handle_function_constants(function_data);
        reports.append(&mut type_reduction::reduce_function(function_data,&program_archive2));
        reports.append(&mut constant_handler_reports);
    }
}

fn bus_level_analyses(program_archive: &ProgramArchive, reports: &mut ReportCollection) {
    let function_names = program_archive.get_function_names();
    for bus_data in program_archive.get_buses().values() {
        let free_of_invalid_statements_result = free_of_invalid_statements(bus_data, function_names);
        if let Result::Err(mut free_of_invalid_statements_reports) = free_of_invalid_statements_result {
            reports.append(&mut free_of_invalid_statements_reports);
        }
    }
}

fn bus_level_decorators(program_archive: &mut ProgramArchive, reports: &mut ReportCollection) {
    let program_archive2 = program_archive.clone();
    for bus_data in program_archive.get_mut_buses().values_mut() {
        let mut constant_handler_reports =
            constants_handler::handle_bus_constants(bus_data);
        reports.append(&mut type_reduction::reduce_bus(bus_data,&program_archive2));
        reports.append(&mut constant_handler_reports);
    }
}

fn semantic_analyses(
    program_archive: &ProgramArchive,
    errors: &mut ReportCollection,
    warnings: &mut ReportCollection,
) {
    for bus_name in program_archive.get_bus_names().iter() {
        if let Result::Err(mut unknown_known_report) =
            unknown_known_analysis(&bus_name, program_archive) {
                errors.append(&mut unknown_known_report);
            }
    }
    for template_name in program_archive.get_template_names().iter() {
        if let Result::Err(mut unknown_known_report) =
            unknown_known_analysis(template_name, program_archive) {
                errors.append(&mut unknown_known_report);
            }
        if program_archive.get_template_data(template_name).is_custom_gate() {
            let body = program_archive.get_template_data(template_name).get_body();
            match custom_gate_analysis(template_name, body) {
                Result::Ok(mut custom_gate_report) => warnings.append(&mut custom_gate_report),
                Result::Err(mut custom_gate_report) => errors.append(&mut custom_gate_report)
            }
        }
    }
}
