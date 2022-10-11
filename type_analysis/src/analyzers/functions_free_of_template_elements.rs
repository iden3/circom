use program_structure::ast::*;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition;
use program_structure::function_data::FunctionData;
use std::collections::HashSet;

pub fn free_of_template_elements(
    function_data: &FunctionData,
    function_names: &HashSet<String>,
) -> Result<(), ReportCollection> {
    let body = function_data.get_body();
    let mut reports = Vec::new();
    analyse_statement(body, function_names, &mut reports);
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

fn analyse_statement(
    stmt: &Statement,
    function_names: &HashSet<String>,
    reports: &mut ReportCollection,
) {
    use Statement::*;
    let file_id = stmt.get_meta().get_file_id();
    match stmt {
        MultSubstitution { .. } => unreachable!(),
        IfThenElse { cond, if_case, else_case, .. } => {
            analyse_expression(cond, function_names, reports);
            analyse_statement(if_case, function_names, reports);
            if let Option::Some(else_block) = else_case {
                analyse_statement(else_block, function_names, reports);
            }
        }
        While { cond, stmt, .. } => {
            analyse_expression(cond, function_names, reports);
            analyse_statement(stmt, function_names, reports);
        }
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                analyse_statement(stmt, function_names, reports);
            }
        }
        InitializationBlock { meta, xtype, initializations } => {
            if let VariableType::Signal(..) = xtype {
                let mut report = Report::error(
                    "Template elements declared inside the function".to_string(),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id, "Declaring template element".to_string());
                reports.push(report);
                return;
            }
            for initialization in initializations.iter() {
                analyse_statement(initialization, function_names, reports);
            }
        }
        Declaration { meta, xtype, dimensions, .. } => {
            if let VariableType::Var = xtype {
                for dimension in dimensions.iter() {
                    analyse_expression(dimension, function_names, reports);
                }
            } else {
                let mut report = Report::error(
                    "Template elements declared inside the function".to_string(),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id, "Declaring template element".to_string());
                reports.push(report);
            }
        }
        Substitution { meta, op, access, rhe, .. } => {
            if op.is_signal_operator() {
                let mut report = Report::error(
                    "Function uses template operators".to_string(),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id, "Template operator found".to_string());
                reports.push(report);
            }
            analyse_access(access, meta, function_names, reports);
            analyse_expression(rhe, function_names, reports);
        }
        ConstraintEquality { meta, lhe, rhe, .. } => {
            let mut report = Report::error(
                format!("Function uses template operators"),
                ReportCode::UndefinedFunction,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id.clone(), format!("Template operator found"));
            reports.push(report);
            analyse_expression(lhe, function_names, reports);
            analyse_expression(rhe, function_names, reports);
        }
        LogCall { args, .. } => {
            for logarg in args {
                if let LogArgument::LogExp(arg) = logarg {
                    analyse_expression(arg, function_names, reports);
                }
            }
        }
        Assert { arg, .. } => {
            analyse_expression(arg, function_names, reports);
        }
        Return { value, .. } => {
            analyse_expression(value, function_names, reports);
        }
    }
}

fn analyse_access(
    access: &Vec<Access>,
    meta: &Meta,
    function_names: &HashSet<String>,
    reports: &mut ReportCollection,
) {
    let file_id = meta.get_file_id();
    for acc in access.iter() {
        if let Access::ArrayAccess(index) = acc {
            analyse_expression(index, function_names, reports);
        } else {
            let mut report = Report::error(
                format!("Function uses component operators"),
                ReportCode::UndefinedFunction,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id.clone(), format!("Template operator found"));
            reports.push(report);
        }
    }
}

fn analyse_expression(
    expr: &Expression,
    function_names: &HashSet<String>,
    reports: &mut ReportCollection,
) {
    use Expression::*;
    let file_id = expr.get_meta().get_file_id();
    match expr {
        InfixOp { lhe, rhe, .. } => {
            analyse_expression(lhe, function_names, reports);
            analyse_expression(rhe, function_names, reports);
        }
        PrefixOp { rhe, .. } => {
            analyse_expression(rhe, function_names, reports);
        }
        ParallelOp{ rhe, ..} =>{
            analyse_expression(rhe, function_names, reports);
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            analyse_expression(cond, function_names, reports);
            analyse_expression(if_true, function_names, reports);
            analyse_expression(if_false, function_names, reports);
        }
        Variable { meta, access, .. } => analyse_access(access, meta, function_names, reports),
        Number(..) => {}
        Call { meta, id, args, .. } => {
            if !function_names.contains(id) {
                let mut report = Report::error(
                    format!("Unknown call in function"),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id.clone(), format!("Is not a function call"));
                reports.push(report);
            }
            for arg in args.iter() {
                analyse_expression(arg, function_names, reports);
            }
        }
        ArrayInLine { values, .. } => {
            for value in values.iter() {
                analyse_expression(value, function_names, reports);
            }
        }
        UniformArray {value, dimension, .. } => {
            analyse_expression(value, function_names, reports);
            analyse_expression(dimension, function_names, reports);


        }
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}
