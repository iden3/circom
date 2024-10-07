use program_structure::ast::*;
use program_structure::environment::CircomEnvironment;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::{generate_file_location, FileID};
use program_structure::program_archive::ProgramArchive;
use std::collections::HashSet;
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
    tags_modified: bool,
    signals_declared: bool,
    modified_variables: HashSet<String>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Tag {
    Known,
    Unknown,
}

// For the vars we store the information:  (is_it_known, is_it_array)
// in case it is an array, if it becomes unknown it will always be unknown
// but it will not generate an error
// Example:
//     a[0] = 0; // a[0] is known
//     a[1] = in; // a[1] is unknown
//     if (a[i] == 5){ in === 5;} // we do not know if there is an error here or not
//          --> we cannot detect the error until execution

type Environment = CircomEnvironment<Tag, Tag, (Tag, bool), Tag>;

pub fn unknown_known_analysis(
    name: &str,
    program_archive: &ProgramArchive,
) -> Result<(), ReportCollection> {
    debug_assert!(Tag::Known < Tag::Unknown);
    let mut environment = Environment::new();
    let (body, file_id) = if program_archive.contains_template(name) {
        let template_data = program_archive.get_template_data(name);
        let template_body = template_data.get_body();
        let file_id = template_data.get_file_id();
        for arg in template_data.get_name_of_params() {
            // We do not know if it is an array or not, so we use the most restrictive option
            environment.add_variable(arg, (Tag::Known, true));
        }
        (template_body, file_id)
    } else {
        debug_assert!(program_archive.contains_bus(name));
        let bus_data = program_archive.get_bus_data(name);
        let bus_body = bus_data.get_body();
        let file_id = bus_data.get_file_id();
        for arg in bus_data.get_name_of_params() {
            // We do not know if it is an array or not, so we use the most restrictive option
            environment.add_variable(arg, (Tag::Known, true));
        }
        (bus_body, file_id)
    };


    let entry = EntryInformation { file_id, environment };
    let result = analyze(body, entry);
    if result.reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(result.reports)
    }
}

fn analyze(stmt: &Statement, entry_information: EntryInformation) -> ExitInformation {
    use Statement::*;
    use Tag::*;

    fn iterate_statements(
        stmts: &[Statement],
        mut reports: ReportCollection,
        mut environment: Environment,
        file_id: FileID,
    ) -> (bool, bool, bool, ReportCollection, Environment, HashSet<String>) {
        let mut constraints_declared = false;
        let mut tags_modified = false;
        let mut signals_declared = false;
        let mut modified_variables: HashSet<String> = HashSet::new();
        for stmt in stmts {
            let entry = EntryInformation { file_id, environment };
            let exit = analyze(stmt, entry);
            constraints_declared = constraints_declared || exit.constraints_declared;
            tags_modified = tags_modified || exit.tags_modified;
            signals_declared = signals_declared || exit.signals_declared;
            modified_variables.extend(exit.modified_variables);
            for report in exit.reports {
                reports.push(report);
            }
            environment = exit.environment;
        }
        (constraints_declared, tags_modified, signals_declared, reports, environment, modified_variables)
    }
    let file_id = entry_information.file_id;
    let mut reports = ReportCollection::new();
    let mut environment = entry_information.environment;
    let mut modified_variables = HashSet::new();
    let mut constraints_declared = false;
    let mut tags_modified = false;
    let mut signals_declared = false;
    match stmt {
        Declaration { xtype, name, dimensions, .. } => {
            match xtype {
                VariableType::Var => {
                    let is_array = dimensions.len() > 0;
                    environment.add_variable(name, (Known, is_array));                
                    modified_variables.insert(name.clone());
                }
                VariableType::Signal(..) => {
                    environment.add_intermediate(name, Unknown);
                    signals_declared = true;
                }
                VariableType::Bus(..) => {
                    environment.add_intermediate_bus(name, Unknown);
                    signals_declared = true;
                }
                VariableType::Component
                | VariableType::AnonymousComponent => {
                    environment.add_component(name, Unknown);
                    signals_declared = true;
                }
            }

            match xtype {
                VariableType::AnonymousComponent => {}
                _ => {
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
            }
        }
        Substitution { meta, var, access, op, rhe, .. } => {
            let reduced_type = meta.get_type_knowledge().get_reduces_to();
            let expression_tag = tag(rhe, &environment);
            let mut access_tag = Known;
            for acc in access {
                match acc {
                    Access::ArrayAccess(exp) => {
                        access_tag = tag(exp, &environment);
                    }
                    _ => {}
                }
                if access_tag == Unknown {
                    break;
                }
            }
            match reduced_type {
                TypeReduction::Variable => {
                    let (value, is_array) = environment.get_mut_variable_or_break(var, file!(), line!());
                    if !*is_array { // if it is a single variable we always update
                        *value = max(expression_tag, access_tag);
                    } else if *value == Known{ // if not, if it was ukn it remains ukn
                        *value = max(expression_tag, access_tag);
                    }
                    modified_variables.insert(var.clone());

                }
                TypeReduction::Component(_) => {
                    constraints_declared = true;
                    if expression_tag == Unknown {
                        add_report(ReportCode::UnknownTemplate, rhe.get_meta(), file_id, &mut reports);
                    }
                    if access_tag == Unknown {
                        add_report(ReportCode::UnknownTemplate, meta, file_id, &mut reports);
                    }
                }
                TypeReduction::Bus(_) => {
                    if  *op == AssignOp::AssignVar && expression_tag == Unknown {
                        add_report(ReportCode::UnknownBus, meta, file_id, &mut reports);
                    }
                    if *op == AssignOp::AssignConstraintSignal {
                        constraints_declared = true;
                        if is_non_quadratic(rhe, &environment) {
                            add_report(ReportCode::NonQuadratic, rhe.get_meta(), file_id, &mut reports);
                        }
                        if access_tag == Unknown {
                            add_report(ReportCode::NonQuadratic, meta, file_id, &mut reports);
                        }
                    }
                }
                TypeReduction::Tag => {
                    tags_modified = true;
                    if expression_tag == Unknown {
                        add_report(ReportCode::NonValidTagAssignment, rhe.get_meta(), file_id, &mut reports);
                    }
                    if access_tag == Unknown {
                        add_report(ReportCode::NonValidTagAssignment, meta, file_id, &mut reports);
                    }
                }
                _ => {
                    if *op == AssignOp::AssignConstraintSignal {
                        constraints_declared = true;
                        if is_non_quadratic(rhe, &environment) {
                            add_report(ReportCode::NonQuadratic, rhe.get_meta(), file_id, &mut reports);
                        }
                        if access_tag == Unknown {
                            add_report(ReportCode::NonQuadratic, meta, file_id, &mut reports);
                        }
                    }
                    else if environment.has_component(var){
                        if access_tag == Unknown {
                            add_report(ReportCode::UnknownTemplateAssignment, meta, file_id, &mut reports);
                        }
                    }
                }
            }
        }
        UnderscoreSubstitution { op, rhe, .. } => {
            let _expression_tag = tag(rhe, &environment);
            if *op == AssignOp::AssignConstraintSignal {
                constraints_declared = true;
                if is_non_quadratic(rhe, &environment) {
                    add_report(ReportCode::NonQuadratic, rhe.get_meta(), file_id, &mut reports);
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
                    modified_variables : HashSet::new(),
                    tags_modified : false,
                    signals_declared: false,
                }
            };
            constraints_declared =
                else_case_info.constraints_declared || if_case_info.constraints_declared;
            tags_modified = else_case_info.tags_modified || if_case_info.tags_modified;
            signals_declared = else_case_info.signals_declared || if_case_info.signals_declared;
            modified_variables.extend(if_case_info.modified_variables);
            modified_variables.extend(else_case_info.modified_variables);
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
            if tag_cond == Unknown{
                for var in &modified_variables{
                    if environment.has_variable(var){
                        let (value, _is_array) = environment.get_mut_variable_or_break(var, file!(), line!());
                        *value = Unknown;
                    }
                }

            }
            if tag_cond == Unknown && constraints_declared {
                add_report(
                    ReportCode::UnreachableConstraints,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
            if tag_cond == Unknown && tags_modified {
                add_report(
                    ReportCode::UnreachableTags,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
            if tag_cond == Unknown && signals_declared {
                add_report(
                    ReportCode::UnreachableSignals,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
        }
        While { cond, stmt, .. } => {
            let mut entry_info = environment.clone();
            let mut entry = EntryInformation { file_id, environment};
            let mut exit = analyze(stmt, entry);
            let mut modified = check_modified(entry_info, &mut exit.environment, &exit.modified_variables);
            environment = exit.environment;
            while modified{
                entry_info = environment.clone();
                entry = EntryInformation { file_id, environment};
                exit = analyze(stmt, entry);
                modified = check_modified(entry_info, &mut exit.environment, &exit.modified_variables);
                environment = exit.environment;
            };

            constraints_declared = exit.constraints_declared;
            tags_modified = exit.tags_modified;
            signals_declared = exit.signals_declared;
            for report in exit.reports {
                reports.push(report);
            }
            let tag_out = tag(cond, &environment);

            if tag_out == Unknown{
                for var in &exit.modified_variables{
                    if environment.has_variable(var){
                        let (value, _is_array) = environment.get_mut_variable_or_break(var, file!(), line!());
                        *value = Unknown;
                    }
                }   
            }

            if constraints_declared && tag_out == Unknown {
                add_report(
                    ReportCode::UnreachableConstraints,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
            if tag_out == Unknown && tags_modified {
                add_report(
                    ReportCode::UnreachableTags,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
            if tag_out == Unknown && signals_declared {
                add_report(
                    ReportCode::UnreachableSignals,
                    cond.get_meta(),
                    file_id,
                    &mut reports,
                );
            }
        }
        Block { stmts, .. } => {
            environment.add_variable_block();
            let (nc, tags, ns, nr, ne, nm) = iterate_statements(stmts, reports, environment, file_id);
            constraints_declared = nc;
            reports = nr;
            environment = ne;
            modified_variables = nm;
            environment.remove_variable_block();
            tags_modified = tags;
            signals_declared = ns;
        }
        InitializationBlock { initializations, .. } => {
            let (nc, tags, ns, nr, ne, nm) = iterate_statements(initializations, reports, environment, file_id);
            constraints_declared = nc;
            reports = nr;
            environment = ne;
            modified_variables = nm;
            tags_modified = tags;
            signals_declared = ns;
        }
        _ => {}
    }
    ExitInformation { 
        reports, environment, constraints_declared, modified_variables, tags_modified, signals_declared
    }
}

fn tag(expression: &Expression, environment: &Environment) -> Tag {
    use Expression::*;
    use Tag::*;
    match expression {
        Number(_, _) => Known,
        Variable { meta, name,.. } => {
            let reduced_type = meta.get_type_knowledge().get_reduces_to();
            match reduced_type {
                TypeReduction::Variable => {
                    let (tag, is_array) = environment.get_variable_or_break(name, file!(), line!());
                    if *is_array{
                        Known
                    } else{
                        *tag
                    }
                },
                TypeReduction::Signal => Unknown,
                TypeReduction::Bus(_) => Unknown,
                TypeReduction::Component(_) => *environment.get_component_or_break(name, file!(), line!()),
                TypeReduction::Tag => Known,
            }
        }
        ArrayInLine { values, .. } 
        | Call { args: values, .. } 
        | BusCall { args: values, .. }=> {
            expression_iterator(values, Known, Unknown, environment)
        }
        UniformArray { value, dimension, .. } => {
            let tag_value = tag(value, environment);
            let tag_dimension = tag(dimension, environment);
            max(tag_value, tag_dimension)
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
        ParallelOp { rhe, .. } => tag(rhe, environment),
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}
// ***************************** Compare two variable states ********************

fn check_modified(
    initial_state: Environment,
    final_state: &mut Environment,
    modified_variables: &HashSet<String>,
) -> bool {
    let mut modified = false;
    for v in modified_variables {
        if initial_state.has_variable(v) && final_state.has_variable(v) {
            let t_ini = initial_state.get_variable_or_break(v, file!(), line!());
            let t_fin = final_state.get_mut_variable_or_break(v, file!(), line!());
            if *t_ini != *t_fin{
                if t_fin.0 == Tag::Unknown{ // in other case we can enter in loops
                    modified = true;
                }
                *t_fin = max(*t_ini, *t_fin);
            }
        }
    }
    modified
}

// ****************************** Expression utils ******************************
fn expression_iterator(
    values: &[Expression],
    end_tag: Tag,
    look_for: Tag,
    environment: &Environment,
) -> Tag {
    for value in values {
        let index_tag = tag(value, environment);
        if index_tag == look_for {
            return look_for;
        }
    }
    end_tag
}

//  ****************************** Early non-quadratic detection ******************************

fn is_non_quadratic(exp: &Expression, environment: &Environment) -> bool {
    unknown_index(exp, environment)
}

fn unknown_index(exp: &Expression, environment: &Environment) -> bool {
    use Expression::*;
    use Tag::*;
    match exp {
        Number(..) => false,
        Variable { access, .. } => {
            let mut has_unknown_index = false;
            for acc in access {
                if let Access::ArrayAccess(ex) = acc {
                    has_unknown_index = Unknown == tag(ex, environment);
                }
                if has_unknown_index {
                    break;
                }
            }
            has_unknown_index
        }
        InfixOp { lhe, rhe, .. } => {
            unknown_index(lhe.as_ref(), environment) || unknown_index(rhe.as_ref(), environment)
        }
        PrefixOp { rhe, .. } => {
            unknown_index(rhe.as_ref(), environment)
        }
        ParallelOp { rhe, .. } => {
            unknown_index(rhe.as_ref(), environment)
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            unknown_index(cond.as_ref(), environment)
            || unknown_index(if_true.as_ref(), environment)
            || unknown_index(if_false.as_ref(), environment)
        }
        Call { args: exprs, .. }
        | BusCall { args: exprs, .. }
        | ArrayInLine { values: exprs, .. }
        | Tuple { values: exprs, .. } => {
            let mut has_unknown_index = false;
            for exp in exprs {
                has_unknown_index = has_unknown_index || unknown_index(exp, environment);
                if has_unknown_index {
                    break;
                }
            }
            has_unknown_index
        }
        UniformArray{ value, dimension, .. } => {
            unknown_index(value.as_ref(), environment) || unknown_index(dimension.as_ref(), environment)
        }
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
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
        UnknownTemplateAssignment => "Assigments to signals within an unknown access to an array of components are not allowed".to_string(),
        UnknownBus => "Parameters of a bus must be known during the constraint generation phase".to_string(),
        UnknownDimension => "The length of every array must be known during the constraint generation phase".to_string(),
        UnknownTemplate => "Every component instantiation must be resolved during the constraint generation phase. This component declaration uses a value that can be unknown during the constraint generation phase.".to_string(),
        NonValidTagAssignment => "Tags cannot be assigned to values that can be unknown during the constraint generation phase".to_string(),
        NonQuadratic => "Non-quadratic constraint was detected statically, using unknown index will cause the constraint to be non-quadratic".to_string(),
        UnreachableConstraints => "There are constraints depending on the value of the condition and it can be unknown during the constraint generation phase".to_string(),
        UnreachableTags => "There are tag assignments depending on the value of the condition and it can be unknown during the constraint generation phase".to_string(),
        UnreachableSignals => "There are signal, bus or component declarations depending on the value of the condition and it can be unknown during the constraint generation phase".to_string(),
        _ => panic!("Unimplemented error code")
    };
    report.add_primary(location, file_id, message);
    reports.push(report);
}