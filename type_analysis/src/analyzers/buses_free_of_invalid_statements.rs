use program_structure::ast::*;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition;
use program_structure::bus_data::BusData;
use std::collections::HashSet;

pub fn free_of_invalid_statements(
    bus_data: &BusData,
    bus_names: &HashSet<String>,
) -> Result<(), ReportCollection> {
    let body = bus_data.get_body();
    let mut reports = Vec::new();
    analyse_statement(body, bus_names, &mut reports);
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

fn analyse_statement(
    stmt: &Statement,
    bus_names: &HashSet<String>,
    reports: &mut ReportCollection,
) {
    use Statement::*;
    let file_id = stmt.get_meta().get_file_id();
    match stmt {
        MultSubstitution { .. } => unreachable!(),
        IfThenElse { .. } => {
            let mut report = Report::error(
                "Conditional statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
        While { .. } => {
            let mut report = Report::error(
                "Loop statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                analyse_statement(stmt, bus_names, reports);
            }
        },
        InitializationBlock { meta, xtype, initializations } => {
            let mut report = Report::error(
                "Initialization statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
        Declaration { meta, xtype, dimensions, .. } => {
            match xtype {
                VariableType::Signal(stype, tag_list) if stype == ast::SignalType::Intermediate |
                VariableType::Bus(stype, tag_list) if stype == ast::SignalType::Intermediate => {
                    for dimension in dimensions.iter() {
                        analyse_expression(dimension, bus_names, reports);
                    }
                },
                _ => {
                    let mut report = Report::error(
                        "Template elements declared inside the bus".to_string(),
                        ReportCode::UndefinedBus,
                    );
                    let location =
                        file_definition::generate_file_location(meta.get_start(), meta.get_end());
                    report.add_primary(location, file_id, "Declaring template element".to_string());
                    reports.push(report);
                }
            }
        },
        Substitution { .. } | UnderscoreSubstitution { .. } => {
            let mut report = Report::error(
                "Substitution statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
        ConstraintEquality { .. } => {
            let mut report = Report::error(
                "Constraint statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
        LogCall { .. } => {
            let mut report = Report::error(
                "I/O statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
        Assert { .. } => {
            let mut report = Report::error(
                "Assert statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
        Return { .. } => {
            let mut report = Report::error(
                "Return statement used inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Using invalid statement".to_string());
            reports.push(report);
        },
    }
}

fn analyse_expression(
    expr: &Expression,
    bus_names: &HashSet<String>,
    reports: &mut ReportCollection,
) {
    use Expression::*;
    let file_id = expr.get_meta().get_file_id();
    match expr {
        InfixOp { lhe, rhe, .. } => {
            analyse_expression(lhe, bus_names, reports);
            analyse_expression(rhe, bus_names, reports);
        }
        PrefixOp { rhe, .. } => {
            analyse_expression(rhe, bus_names, reports);
        }
        ParallelOp{ rhe, ..} =>{
            analyse_expression(rhe, bus_names, reports);
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            analyse_expression(cond, bus_names, reports);
            analyse_expression(if_true, bus_names, reports);
            analyse_expression(if_false, bus_names, reports);
        }
        Variable { meta, access, .. } => analyse_access(access, meta, bus_names, reports),
        Number(..) => {}
        Call { meta, id, args, .. } => {
            if !bus_names.contains(id) {
                let mut report = Report::error(
                    format!("Unknown call in bus"),
                    ReportCode::UndefinedBus,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id.clone(), format!("Is not a function call"));
                reports.push(report);
            }
            for arg in args.iter() {
                analyse_expression(arg, bus_names, reports);
            }
        }
        ArrayInLine { values, .. } => {
            for value in values.iter() {
                analyse_expression(value, bus_names, reports);
            }
        }
        UniformArray {value, dimension, .. } => {
            analyse_expression(value, bus_names, reports);
            analyse_expression(dimension, bus_names, reports);
        }
        BusCall { meta, .. } => {
            let mut report = Report::error(
                "Template elements declared inside the bus".to_string(),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Declaring template element".to_string());
            reports.push(report);
        },
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}

fn analyse_access(
    access: &Vec<Access>,
    meta: &Meta,
    bus_names: &HashSet<String>,
    reports: &mut ReportCollection,
) {
    let file_id = meta.get_file_id();
    for acc in access.iter() {
        if let Access::ArrayAccess(index) = acc {
            analyse_expression(index, bus_names, reports);
        } else {
            let mut report = Report::error(
                format!("Bus uses component operators"),
                ReportCode::UndefinedBus,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id.clone(), format!("Template operator found"));
            reports.push(report);
        }
    }
}