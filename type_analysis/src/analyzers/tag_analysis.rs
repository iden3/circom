// use program_structure::ast::{
//     Access, AssignOp, Expression, ExpressionPrefixOpcode, Meta, SignalElementType, SignalType,
//     Statement, VariableType,
// };
// use program_structure::environment::CircomEnvironment;
// use program_structure::error_code::ReportCode;
// use program_structure::error_definition::{Report, ReportCollection};
// use program_structure::file_definition::{generate_file_location, FileID};
// use program_structure::program_archive::ProgramArchive;
// use program_structure::template_data::TemplateData;
// use std::collections::HashMap;

// type TemplateInfo = HashMap<String, TemplateData>;
// type Environment = CircomEnvironment<Option<String>, SignalElementType, SignalElementType>;
// enum ExpressionResult {
//     ArithmeticExpression(SignalElementType),
//     Template(Option<String>),
// }

// pub fn tag_analysis(
//     template_name: &str,
//     program_archive: &ProgramArchive,
// ) -> Result<(), ReportCollection> {
//     let template_body = program_archive.get_template_data(template_name).get_body_as_vec();
//     let file_id = program_archive.get_template_data(template_name).get_file_id();
//     let template_info = program_archive.get_templates();

//     let mut environment = Environment::new();
//     let args = program_archive.get_template_data(template_name).get_name_of_params().clone();
//     for arg in args.iter() {
//         environment.add_variable(arg, SignalElementType::FieldElement);
//     }

//     let mut reports = ReportCollection::new();
//     treat_sequence_of_statements(
//         template_body,
//         file_id,
//         template_info,
//         &mut reports,
//         &mut environment,
//     );
//     if reports.is_empty() {
//         Result::Ok(())
//     } else {
//         Result::Err(reports)
//     }
// }

// fn statement_inspection(
//     stmt: &Statement,
//     file_id: FileID,
//     template_info: &TemplateInfo,
//     reports: &mut ReportCollection,
//     environment: &mut Environment,
// ) {
//     use Statement::*;
//     match stmt {
//         IfThenElse { if_case, else_case, .. } => {
//             statement_inspection(if_case, file_id, template_info, reports, environment);
//             if let Option::Some(else_stmt) = else_case {
//                 statement_inspection(else_stmt, file_id, template_info, reports, environment);
//             }
//         }
//         While { stmt, .. } => {
//             statement_inspection(stmt, file_id, template_info, reports, environment);
//         }
//         Block { stmts, .. } => {
//             environment.add_variable_block();
//             treat_sequence_of_statements(stmts, file_id, template_info, reports, environment);
//             environment.remove_variable_block();
//         }
//         InitializationBlock { initializations, .. } => {
//             treat_sequence_of_statements(
//                 initializations,
//                 file_id,
//                 template_info,
//                 reports,
//                 environment,
//             );
//         }
//         Declaration { xtype, name, meta, .. } => {
//             use SignalType::*;
//             use VariableType::*;
//             match xtype {
//                 Var => environment.add_variable(name, SignalElementType::FieldElement),
//                 Component => environment.add_component(name, meta.component_inference.clone()),
//                 Signal(signal_type, signal_element) => match signal_type {
//                     Output => environment.add_output(name, *signal_element),
//                     Intermediate => environment.add_intermediate(name, *signal_element),
//                     Input => environment.add_input(name, *signal_element),
//                 },
//             }
//         }
//         Substitution { meta, var, access, rhe, op, .. } => {
//             use ExpressionResult::*;
//             let var_info = variable_inspection(var, access, environment, template_info);
//             let rhe_info = expression_inspection(rhe, template_info, reports, environment);
//             match (var_info, rhe_info) {
//                 (Template(_), Template(assign)) => {
//                     *environment.get_mut_component_or_break(var, file!(), line!()) = assign;
//                 }
//                 (ArithmeticExpression(tag_0), ArithmeticExpression(tag_1))
//                     if tag_0 == SignalElementType::Binary
//                         && tag_1 == SignalElementType::FieldElement
//                         && *op == AssignOp::AssignConstraintSignal =>
//                 {
//                     add_report(ReportCode::WrongSignalTags, meta, file_id, reports)
//                 }
//                 _ => {}
//             }
//         }
//         _ => {}
//     }
// }

      
// fn expression_inspection(
//     expr: &Expression,
//     template_info: &TemplateInfo,
//     reports: &mut ReportCollection,
//     environment: &Environment,
// ) -> ExpressionResult {
//     use Expression::*;
//     match expr {
//         InfixOp { .. } => ExpressionResult::ArithmeticExpression(SignalElementType::FieldElement),
//         PrefixOp { rhe, prefix_op, .. } => {
//             let rhe_info = expression_inspection(rhe, template_info, reports, environment);
//             match prefix_op {
//                 ExpressionPrefixOpcode::BoolNot => rhe_info,
//                 _ => ExpressionResult::ArithmeticExpression(SignalElementType::FieldElement),
//             }
//         }
//         InlineSwitchOp { if_true, if_false, .. } => {
//             let if_true_info = expression_inspection(if_true, template_info, reports, environment);
//             let if_false_info =
//                 expression_inspection(if_false, template_info, reports, environment);
//             match (&if_true_info, &if_false_info) {
//                 (
//                     ExpressionResult::ArithmeticExpression(tag_0),
//                     ExpressionResult::ArithmeticExpression(tag_1),
//                 ) => {
//                     if let SignalElementType::FieldElement = tag_0 {
//                         if_true_info
//                     } else if let SignalElementType::FieldElement = tag_1 {
//                         if_false_info
//                     } else {
//                         if_true_info
//                     }
//                 }
//                 _ => if_true_info,
//             }
//         }
//         Variable { name, access, .. } => {
//             variable_inspection(name, access, environment, template_info)
//         }
//         Number(..) => ExpressionResult::ArithmeticExpression(SignalElementType::FieldElement),
//         Call { id, .. } => {
//             if template_info.contains_key(id) {
//                 ExpressionResult::Template(Option::Some(id.clone()))
//             } else {
//                 ExpressionResult::ArithmeticExpression(SignalElementType::FieldElement)
//             }
//         }
//         ArrayInLine { .. } => {
//             ExpressionResult::ArithmeticExpression(SignalElementType::FieldElement)
//         }
//         UniformArray { .. } => {
//             ExpressionResult::ArithmeticExpression(SignalElementType::FieldElement)
//         }
//         _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
//     }
// }

// //************************************************* Statement support *************************************************
// fn treat_sequence_of_statements(
//     stmts: &[Statement],
//     file_id: FileID,
//     template_info: &TemplateInfo,
//     reports: &mut ReportCollection,
//     environment: &mut Environment,
// ) {
//     for stmt in stmts {
//         statement_inspection(stmt, file_id, template_info, reports, environment);
//     }
// }
// //************************************************* Expression support *************************************************

// fn variable_inspection(
//     symbol: &str,
//     accesses: &[Access],
//     environment: &Environment,
//     template_info: &TemplateInfo,
// ) -> ExpressionResult {
//     use ExpressionResult::*;
//     let mut result = if environment.has_component(symbol) {
//         Template(environment.get_component_or_break(symbol, file!(), line!()).clone())
//     } else if environment.has_signal(symbol) {
//         ArithmeticExpression(*environment.get_signal_or_break(symbol, file!(), line!()))
//     } else {
//         ArithmeticExpression(SignalElementType::FieldElement)
//     };

//     for access in accesses {
//         if let Access::ComponentAccess(signal) = access {
//             let template =
//                 environment.get_component_or_break(symbol, file!(), line!()).clone().unwrap();
//             let input = template_info.get(&template).unwrap().get_input_info(signal);
//             let output = template_info.get(&template).unwrap().get_output_info(signal);
//             match (input, output) {
//                 (Some((_, tag)), _) | (_, Some((_, tag))) => {
//                     result = ArithmeticExpression(*tag);
//                 }
//                 _ => {
//                     unreachable!()
//                 }
//             }
//         }
//     }
//     result
// }

// //************************************************* Report support *************************************************
// fn add_report(
//     error_code: ReportCode,
//     meta: &Meta,
//     file_id: FileID,
//     reports: &mut ReportCollection,
// ) {
//     use ReportCode::*;
//     let mut report = Report::error("Typing error found".to_string(), error_code);
//     let location = generate_file_location(meta.start, meta.end);
//     let message = match error_code {
//         WrongSignalTags => "Can not assign Field values to signals tagged as binary".to_string(),
//         _ => panic!("Unimplemented error code"),
//     };
//     report.add_primary(location, file_id, message);
//     reports.push(report);
// }
