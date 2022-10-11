use program_structure::ast::*;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};

pub fn custom_gate_analysis(
    custom_gate_name: &str,
    custom_gate_body: &Statement
) -> Result<ReportCollection, ReportCollection> {
    fn custom_gate_analysis(
        custom_gate_name: &str,
        stmt: &Statement,
        errors: &mut ReportCollection,
        warnings: &mut ReportCollection
    ) {
        use Statement::*;
        match stmt {
            IfThenElse { if_case, else_case, .. } => {
                custom_gate_analysis(custom_gate_name, if_case, warnings, errors);
                if let Some(else_case_s) = else_case {
                    custom_gate_analysis(custom_gate_name, else_case_s, errors, warnings);
                }
            }
            While { stmt, .. } => {
                custom_gate_analysis(custom_gate_name, stmt, errors, warnings);
            }
            InitializationBlock { initializations, .. } => {
                for stmt in initializations {
                    custom_gate_analysis(custom_gate_name, stmt, errors, warnings);
                }
            }
            Declaration { meta, xtype, name, .. } => {
                use VariableType::*;
                match xtype {
                    Signal(SignalType::Intermediate, _) => {
                        let mut warning = Report::warning(
                            String::from("Intermediate signal inside custom template"),
                            ReportCode::CustomGateIntermediateSignalWarning
                        );
                        warning.add_primary(
                            meta.location.clone(),
                            meta.file_id.unwrap(),
                            format!(
                                "Intermediate signal {} declared in custom template {}",
                                name,
                                custom_gate_name
                            )
                        );
                        warnings.push(warning);
                    }
                    Component | AnonymousComponent => {
                        let mut error = Report::error(
                            String::from("Component inside custom template"),
                            ReportCode::CustomGateSubComponentError
                        );
                        error.add_primary(
                            meta.location.clone(),
                            meta.file_id.unwrap(),
                            format!(
                                "Component {} declared in custom template {}",
                                name,
                                custom_gate_name
                            )
                        );
                        errors.push(error);
                    }
                    _ => {}
                }
            }
            Substitution { meta, op, .. } => {
                use AssignOp::*;
                match op {
                    AssignConstraintSignal => {
                        let mut error = Report::error(
                            String::from("Added constraint inside custom template"),
                            ReportCode::CustomGateConstraintError
                        );
                        error.add_primary(
                            meta.location.clone(),
                            meta.file_id.unwrap(),
                            String::from("Added constraint")
                        );
                        errors.push(error);
                    }
                    _ => {}
                }
            }
            ConstraintEquality { meta, .. } => {
                let mut error = Report::error(
                    String::from("Added constraint inside custom template"),
                    ReportCode::CustomGateConstraintError
                );
                error.add_primary(
                    meta.location.clone(),
                    meta.file_id.unwrap(),
                    String::from("Added constraint")
                );
                errors.push(error);
            }
            Block { stmts, .. } => {
                for stmt in stmts {
                    custom_gate_analysis(custom_gate_name, stmt, errors, warnings);
                }
            }
            _ => {}
        };
    }

    let mut warnings = vec![];
    let mut errors = vec![];

    custom_gate_analysis(custom_gate_name, custom_gate_body, &mut warnings, &mut errors);

    if errors.is_empty() {
        Result::Ok(warnings)
    } else {
        Result::Err(errors)
    }
}
