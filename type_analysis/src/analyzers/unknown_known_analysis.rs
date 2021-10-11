use program_structure::ast::*;
use program_structure::environment::CircomEnvironment;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::{generate_file_location, FileID};
use program_structure::program_archive::ProgramArchive;
use std::cmp::max;
use std::option::Option;

struct EntryInformation {
    file_id: FileID,
    environment: Environment,
}
struct ExitInformation {
    reports: ReportCollection,
    environment: Environment,
    constraints_declared: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Tag {
    Known,
    Unknown,
}
type Environment = CircomEnvironment<Tag, Tag, Tag>;

pub fn unknown_known_analysis(
    template_name: &str,
    program_archive: &ProgramArchive,
) -> Result<(), ReportCollection> {
    debug_assert!(Tag::Known < Tag::Unknown);
    let template_data = program_archive.get_template_data(template_name);
    let template_body = template_data.get_body();
    let file_id = template_data.get_file_id();
    let mut environment = Environment::new();
    for arg in template_data.get_name_of_params() {
        environment.add_variable(arg, Tag::Known);
    }

    let entry = EntryInformation { file_id, environment };
    let result = analyze(template_body, entry);
    if result.reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(result.reports)
    }
}

fn analyze(stmt: &Statement, entry_information: EntryInformation) -> ExitInformation {
    use Statement::*;
    use Symbol::*;
    use Tag::*;

    fn iterate_statements(
        stmts: &[Statement],
        mut reports: ReportCollection,
        mut environment: Environment,
        file_id: FileID,
    ) -> (bool, ReportCollection, Environment) {
        let mut constraints_declared = false;
        for stmt in stmts {
            let entry = EntryInformation { file_id, environment };
            let exit = analyze(stmt, entry);
            constraints_declared = constraints_declared || exit.constraints_declared;
            for report in exit.reports {
                reports.push(report);
            }
            environment = exit.environment;
        }
        (constraints_declared, reports, environment)
    }
    let file_id = entry_information.file_id;
    let mut reports = ReportCollection::new();
    let mut environment = entry_information.environment;
    let mut constraints_declared = false;
    match stmt {
        Declaration { xtype, name, dimensions, .. } => {
            if let VariableType::Signal(..) = xtype {
                environment.add_intermediate(name, Unknown)
            } else if let VariableType::Component = xtype {
                environment.add_component(name, Known);
            } else {
                environment.add_variable(name, Known);
            }

            for dimension in dimensions {
                if tag(dimension, &environment) == Unknown {
                    add_report(
                        ReportCode::UnknownDimension,
                        dimension.get_meta(),
                        file_id,
                        &mut reports,
                    );
                }
            }
        }
        Substitution { meta, var, access, op, rhe, .. } => {
            let simplified_elem = simplify_symbol(&environment, var, access);
            let expression_tag = tag(rhe, &environment);
            let mut access_tag = Known;
            for acc in access {
                match acc {
                    Access::ArrayAccess(exp) if access_tag != Unknown => {
                        access_tag = tag(exp, &environment);
                    }
                    _ => {}
                }
            }
            if simplified_elem == Variable {
                let value = environment.get_mut_variable_or_break(var, file!(), line!());
                *value = max(expression_tag, access_tag);
            } else if simplified_elem == Component {
                constraints_declared = true;
                if expression_tag == Unknown {
                    add_report(ReportCode::UnknownTemplate, rhe.get_meta(), file_id, &mut reports);
                }
                if access_tag == Unknown {
                    add_report(ReportCode::UnknownTemplate, meta, file_id, &mut reports);
                }
            } else if *op == AssignOp::AssignConstraintSignal {
                constraints_declared = true;
                if is_non_quadratic(rhe, &environment) {
                    add_report(ReportCode::NonQuadratic, rhe.get_meta(), file_id, &mut reports);
                }
                if access_tag == Unknown {
                    add_report(ReportCode::NonQuadratic, meta, file_id, &mut reports);
                }
            }
        }
        ConstraintEquality { lhe, rhe, .. } => {
            constraints_declared = true;
            if is_non_quadratic(lhe, &environment) {
                add_report(ReportCode::NonQuadratic, lhe.get_meta(), file_id, &mut reports);
            }
            if is_non_quadratic(rhe, &environment) {
                add_report(ReportCode::NonQuadratic, rhe.get_meta(), file_id, &mut reports);
            }
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            let tag_cond = tag(cond, &environment);
            let new_entry_else_case =
                EntryInformation { environment: environment.clone(), file_id };
            let new_entry_if_case = EntryInformation { environment, file_id };
            let if_case_info = analyze(if_case, new_entry_if_case);
            let else_case_info = if let Option::Some(else_stmt) = else_case {
                analyze(else_stmt, new_entry_else_case)
            } else {
                ExitInformation {
                    constraints_declared: false,
                    environment: new_entry_else_case.environment,
                    reports: ReportCollection::with_capacity(0),
                }
            };
            constraints_declared =
                else_case_info.constraints_declared || if_case_info.constraints_declared;
            for report in if_case_info.reports {
                reports.push(report);
            }
            for report in else_case_info.reports {
                reports.push(report);
            }
            environment =
                Environment::merge(if_case_info.environment, else_case_info.environment, |a, b| {
                    max(a, b)
                });
            if tag_cond == Unknown && constraints_declared {
                add_report(
                    ReportCode::UnreachableConstraints,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
        }
        While { cond, stmt, .. } => {
            let tag_entry = tag(cond, &environment);
            let entry = EntryInformation { file_id, environment };
            let exit = analyze(stmt, entry);
            constraints_declared = exit.constraints_declared;
            for report in exit.reports {
                reports.push(report);
            }
            environment = exit.environment;
            let tag_out = tag(cond, &environment);
            if constraints_declared && max(tag_entry, tag_out) == Unknown {
                add_report(
                    ReportCode::UnreachableConstraints,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
        }
        Block { stmts, .. } => {
            environment.add_variable_block();
            let (nc, nr, ne) = iterate_statements(stmts, reports, environment, file_id);
            constraints_declared = nc;
            reports = nr;
            environment = ne;
            environment.remove_variable_block();
        }
        InitializationBlock { initializations, .. } => {
            let (nc, nr, ne) = iterate_statements(initializations, reports, environment, file_id);
            constraints_declared = nc;
            reports = nr;
            environment = ne;
        }
        _ => {}
    }
    ExitInformation { reports, environment, constraints_declared }
}

fn tag(expression: &Expression, environment: &Environment) -> Tag {
    use Expression::*;
    use Tag::*;
    match expression {
        Number(_, _) => Known,
        Variable { name, access, .. } => {
            let mut symbol_tag = if environment.has_variable(name) {
                *environment.get_variable_or_break(name, file!(), line!())
            } else if environment.has_component(name) {
                *environment.get_component_or_break(name, file!(), line!())
            } else {
                *environment.get_intermediate_or_break(name, file!(), line!())
            };
            let mut index = 0;
            loop {
                if index == access.len() {
                    break symbol_tag;
                }
                if symbol_tag == Unknown {
                    break Unknown;
                }
                if let Access::ArrayAccess(exp) = &access[index] {
                    symbol_tag = tag(exp, environment);
                } else {
                    symbol_tag = Unknown;
                }
                index += 1;
            }
        }
        ArrayInLine { values, .. } | Call { args: values, .. } => {
            expression_iterator(values, Known, Unknown, environment)
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            let tag_cond = tag(cond, environment);
            let tag_true = tag(if_true, environment);
            let tag_false = tag(if_false, environment);
            max(tag_cond, max(tag_true, tag_false))
        }
        InfixOp { lhe, rhe, .. } => {
            let tag_lhe = tag(lhe, environment);
            let tag_rhe = tag(rhe, environment);
            max(tag_rhe, tag_lhe)
        }
        PrefixOp { rhe, .. } => tag(rhe, environment),
    }
}

// ****************************** Expression utils ******************************
fn expression_iterator(
    values: &[Expression],
    end_tag: Tag,
    look_for: Tag,
    environment: &Environment,
) -> Tag {
    let mut index = 0;
    loop {
        if index == values.len() {
            break end_tag;
        }
        let index_tag = tag(&values[index], environment);
        if index_tag == look_for {
            break look_for;
        }
        index += 1;
    }
}

//  ****************************** AST simplification utils ******************************
#[derive(Copy, Clone, Eq, PartialEq)]
enum Symbol {
    Signal,
    Component,
    Variable,
}
fn simplify_symbol(environment: &Environment, name: &str, access: &[Access]) -> Symbol {
    use Symbol::*;
    if environment.has_variable(name) {
        Variable
    } else if environment.has_signal(name) {
        Signal
    } else {
        let mut symbol = Component;
        for acc in access {
            if let Access::ComponentAccess(_) = acc {
                symbol = Signal;
            }
        }
        symbol
    }
}

//  ****************************** Early non-quadratic detection ******************************

fn is_non_quadratic(exp: &Expression, environment: &Environment) -> bool {
    unknown_index(exp, environment)
}

fn unknown_index(exp: &Expression, environment: &Environment) -> bool {
    use Expression::*;
    use Tag::*;
    let (init, rec) = match exp {
        Number(..) => (false, vec![]),
        Variable { access, .. } => {
            let mut has_unknown_index = false;
            let mut index = 0;
            loop {
                if index == access.len() || has_unknown_index {
                    break (has_unknown_index, vec![]);
                }
                if let Access::ArrayAccess(ex) = &access[index] {
                    has_unknown_index = Unknown == tag(ex, environment);
                }
                index += 1;
            }
        }
        InfixOp { lhe, rhe, .. } => (false, vec![lhe.as_ref(), rhe.as_ref()]),
        PrefixOp { rhe, .. } => (false, vec![rhe.as_ref()]),
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            (false, vec![cond.as_ref(), if_true.as_ref(), if_false.as_ref()])
        }
        Call { args: exprs, .. } | ArrayInLine { values: exprs, .. } => {
            let mut bucket = Vec::new();
            for exp in exprs {
                bucket.push(exp);
            }
            (false, bucket)
        }
    };
    let mut has_unknown_index = init;
    let mut index = 0;
    loop {
        if index == rec.len() || has_unknown_index {
            break has_unknown_index;
        }
        has_unknown_index = unknown_index(&rec[index], environment);
        index += 1;
    }
}

// ****************************** Report utils ******************************
fn add_report(
    error_code: ReportCode,
    meta: &Meta,
    file_id: FileID,
    reports: &mut ReportCollection,
) {
    use ReportCode::*;
    let mut report = Report::error("Typing error found".to_string(), error_code);
    let location = generate_file_location(meta.start, meta.end);
    let message = match error_code {
        UnknownDimension => "The length of every array must known during the constraint generation phase".to_string(),
        UnknownTemplate => "Every component instantiation must be resolved during the constraint generation phase".to_string(),
        NonQuadratic => "Non-quadratic constraint was detected statically, using unknown index will cause the constraint to be non-quadratic".to_string(),
        UnreachableConstraints => "There are constraints depending on the value of the condition and it can be unknown during the constraint generation phase".to_string(),
        _ => panic!("Unimplemented error code")
    };
    report.add_primary(location, file_id, message);
    reports.push(report);
}
