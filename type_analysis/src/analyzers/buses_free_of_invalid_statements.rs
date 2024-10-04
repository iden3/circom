use program_structure::ast::*;
use program_structure::program_library::bus_data::BusData;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition;
use std::collections::HashSet;

pub fn free_of_invalid_statements(
    bus_data: &BusData,
    function_names: &HashSet<String>,
) -> Result<(), ReportCollection> {
    let body = bus_data.get_body();
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
        IfThenElse { meta, .. } => {
            report_undefined_bus_error(meta, "Conditional statement used inside the bus",file_id, None, reports);
        },
        While { meta, .. } => {
            report_undefined_bus_error(meta, "Loop statement used inside the bus", file_id, None, reports);
        },
        Block { stmts, .. }  => {
            for stmt in stmts.iter() {
                analyse_statement(stmt, function_names, reports);
            }
        },
        InitializationBlock { initializations, .. } => {
            for stmt in initializations.iter() {
                analyse_statement(stmt, function_names, reports);
            }
        },
        Declaration { meta, xtype, dimensions, .. } => {
            match xtype {
                VariableType::Signal(stype, _)
                | VariableType::Bus(_, stype, _) => {
                    if *stype == SignalType::Intermediate {
                        for dimension in dimensions.iter() {
                            analyse_expression(dimension, function_names, reports);
                        }
                    } else {
                        report_undefined_bus_error(meta, "Template elements declared inside the bus", file_id, None, reports)
                    }
                },
                _ => {
                    report_undefined_bus_error(meta, "Template elements declared inside the bus", file_id, None, reports)
                }
            }
        },
        Substitution { meta, rhe, access, .. } => {
            if !rhe.is_bus_call_array() {
                report_undefined_bus_error(meta, "Substitution statement used inside the bus", file_id, None, reports);
            }
            analyse_access(access, meta, function_names, reports)
        }
        UnderscoreSubstitution { meta, rhe,  .. } => {
            if !rhe.is_bus_call_array() {
                report_undefined_bus_error(meta, "Substitution statement used inside the bus", file_id, None, reports);
            }
        }
        ConstraintEquality { meta, .. } => {
            report_undefined_bus_error(meta, "Constraint statement used inside the bus", file_id, None, reports);
        },
        LogCall { meta, .. } => {
            report_undefined_bus_error(meta, "I/O statement used inside the bus", file_id, None, reports)
        },
        Assert { meta, .. } => {
            report_undefined_bus_error(meta, "Assert statement used inside the bus", file_id, None, reports)
        },
        Return { meta, .. } => {
            report_undefined_bus_error(meta, "Return statement used inside the bus", file_id, None, reports)
        },
    }
}

fn report_undefined_bus_error(meta: &Meta, msg : &str, file_id: usize, primary_msg : Option<&str>, reports: &mut Vec<Report>) {
    let mut report = Report::error(
        msg.to_string(),
        ReportCode::UndefinedBus,
    );
    let location =
        file_definition::generate_file_location(meta.get_start(), meta.get_end());
    let primary_msg = if let Some(msg) = primary_msg  { msg } else  {"Using invalid statement"};
    report.add_primary(location, file_id, primary_msg.to_string());
    reports.push(report);
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
                report_undefined_bus_error(meta, "Unknown call in bus", file_id, Some("Is not a function call"), reports)
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
        BusCall { meta, .. } => {
            report_undefined_bus_error(meta, "Template elements declared inside the bus", file_id, Some("Declaring template element"), reports)
        },
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
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
            report_undefined_bus_error(meta, "Bus uses name-access operators", file_id, Some("Template operator found"), reports)
        }
    }
}