use program_structure::ast::Expression::Call;
use super::type_given_function::type_given_function;
use super::type_register::TypeRegister;
use program_structure::ast::*;
use program_structure::environment::CircomEnvironment;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::{generate_file_location, FileID};
use program_structure::program_archive::ProgramArchive;
use std::collections::HashSet;

type ArithmeticType = usize;
type ComponentInfo = (Option<String>, ArithmeticType);
type TypingEnvironment = CircomEnvironment<ComponentInfo,  (ArithmeticType, std::vec::Vec<std::string::String>), ArithmeticType>;
type CallRegister = TypeRegister<ArithmeticType>;

struct AnalysisInformation {
    file_id: FileID,
    reached: HashSet<String>,
    reports: ReportCollection,
    registered_calls: CallRegister,
    environment: TypingEnvironment,
    return_type: Option<ArithmeticType>,
}

struct FoldedType {
    arithmetic: Option<ArithmeticType>,
    template: Option<String>,
}
impl FoldedType {
    pub fn arithmetic_type(dimensions: ArithmeticType) -> FoldedType {
        FoldedType { arithmetic: Option::Some(dimensions), template: Option::None }
    }
    pub fn template(name: &str) -> FoldedType {
        FoldedType { template: Option::Some(name.to_string()), arithmetic: Option::None }
    }
    pub fn is_template(&self) -> bool {
        self.template.is_some() && self.arithmetic.is_none()
    }
    pub fn dim(&self) -> usize {
        if let Option::Some(dim) = &self.arithmetic {
            *dim
        } else {
            0
        }
    }
    pub fn same_type(left: &FoldedType, right: &FoldedType) -> bool {
        let mut equal = false;
        if let (Option::Some(l_template), Option::Some(r_template)) =
            (&left.template, &right.template)
        {
            equal = l_template.eq(r_template);
        }
        if let (Option::Some(l_dim), Option::Some(r_dim)) = (&left.arithmetic, &right.arithmetic) {
            equal = *l_dim == *r_dim;
        }
        equal
    }
}

pub struct OutInfo {
    pub reached: HashSet<String>,
}

pub fn type_check(program_archive: &ProgramArchive) -> Result<OutInfo, ReportCollection> {
    let mut analysis_information = AnalysisInformation {
        reached: HashSet::new(),
        file_id: *program_archive.get_file_id_main(),
        reports: ReportCollection::new(),
        registered_calls: CallRegister::new(),
        environment: TypingEnvironment::new(),
        return_type: Option::None,
    };
    let initial_expression = program_archive.get_main_expression();
    let type_analysis_response =
        type_expression(initial_expression, program_archive, &mut analysis_information);
    let first_type = if let Result::Ok(t) = type_analysis_response {
        t
    } else {
        return Result::Err(analysis_information.reports);
    };
    if !first_type.is_template() {
        add_report(
            ReportCode::WrongTypesInAssignOperation,
            initial_expression.get_meta(),
            &mut analysis_information.reports,
        );
    }

    if check_main_has_tags(initial_expression, program_archive) {
            add_report(
                ReportCode::MainComponentWithTags,
                initial_expression.get_meta(),
                &mut analysis_information.reports,
            );
    }


    if analysis_information.reports.is_empty() {
        Result::Ok(OutInfo { reached: analysis_information.reached })
    } else {
        Result::Err(analysis_information.reports)
    }
}

fn check_main_has_tags(initial_expression: &Expression, program_archive: &ProgramArchive) -> bool {
    if let  Call { id, .. } = initial_expression{
        let inputs = program_archive.get_template_data(id).get_inputs();
        let mut tag_in_inputs = false;
        for input in inputs {
            if !input.1.1.is_empty(){
                tag_in_inputs = true;
                break;
            }
        }
        tag_in_inputs
    }
    else { unreachable!()}
}

fn type_statement(
    statement: &Statement,
    program_archive: &ProgramArchive,
    analysis_information: &mut AnalysisInformation,
) {
    use Statement::*;
    match statement {
        InitializationBlock { initializations, .. } => {
            treat_sequence_of_statements(initializations, program_archive, analysis_information)
        }
        Declaration { xtype, name, dimensions, meta, .. } => {
            let typing_response =
                type_array_of_expressions(dimensions, program_archive, analysis_information);
            let dimensions_type = typing_response.unwrap_or_default();
            for (dim_expression, dim_type) in dimensions.iter().zip(dimensions_type) {
                if dim_type.is_template() || dim_type.dim() > 0 {
                    add_report(
                        ReportCode::InvalidArraySize,
                        dim_expression.get_meta(),
                        &mut analysis_information.reports,
                    );
                }
            }
            match xtype {
                VariableType::Signal(s_type, tags) => {
                    if let SignalType::Input = s_type {
                        analysis_information.environment.add_input(name, (dimensions.len(),tags.clone()));
                    } else if let SignalType::Output = s_type {
                        analysis_information.environment.add_output(name, (dimensions.len(),tags.clone()));
                    } else {
                        analysis_information.environment.add_intermediate(name, (dimensions.len(),tags.clone()));
                    }
                }
                VariableType::Var => {
                    analysis_information.environment.add_variable(name, dimensions.len())
                }
                VariableType::Component => analysis_information
                    .environment
                    .add_component(name, (meta.component_inference.clone(), dimensions.len())),
                VariableType::AnonymousComponent => analysis_information
                    .environment
                    .add_component(name, (meta.component_inference.clone(), dimensions.len())),
            }
        }
        Substitution { var, access, op, rhe, meta, .. } => {
            let rhe_response = type_expression(rhe, program_archive, analysis_information);
            let rhe_type = if let Result::Ok(r_type) = rhe_response {
                r_type
            } else {
                return;
            };

            let access_information_result =
                treat_access(var, access, meta, program_archive, analysis_information);

            let access_information = if let Result::Ok(info) = access_information_result {
                info
            } else {
                return;
            };

            if analysis_information.environment.has_component(var) && access_information.2.is_some(){
                return add_report(
                    ReportCode::OutputTagCannotBeModifiedOutside,
                    meta,
                    &mut analysis_information.reports,
                );
            }

            let symbol_type_result = apply_access_to_symbol(
                var,
                meta,
                access_information,
                &analysis_information.environment,
                &mut analysis_information.reports,
                program_archive,
            );
            let symbol_information = if let Result::Ok(s_type) = symbol_type_result {
                s_type
            } else {
                return;
            };

            match (&symbol_information, op) {
                (SymbolInformation::Signal(_), AssignOp::AssignConstraintSignal)
                | (SymbolInformation::Signal(_), AssignOp::AssignSignal)
                | (SymbolInformation::Var(_), AssignOp::AssignVar)
                | (SymbolInformation::Component(_), AssignOp::AssignVar) => {}
                | (SymbolInformation::Tag, AssignOp::AssignVar) => {}
                _ => {
                    return add_report(
                        ReportCode::WrongTypesInAssignOperation,
                        meta,
                        &mut analysis_information.reports,
                    );
                }
            }
            match symbol_information {
                SymbolInformation::Component(template)
                    if template.is_none() && rhe_type.is_template() =>
                {
                    let (current_template, _) = analysis_information
                        .environment
                        .get_mut_component_or_break(var, file!(), line!());
                    *current_template = rhe_type.template;
                }
                SymbolInformation::Component(possible_template)
                    if possible_template.is_some() && rhe_type.is_template() =>
                {
                    let template = possible_template.unwrap();
                    let r_template = rhe_type.template.unwrap();
                    if template != r_template {
                        add_report(
                            ReportCode::WrongTypesInAssignOperation,
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                }
                SymbolInformation::Signal(dim)
                    if dim == rhe_type.dim() && !rhe_type.is_template() => {}
                SymbolInformation::Var(dim)
                    if dim == rhe_type.dim() && !rhe_type.is_template() => {}
                SymbolInformation::Tag if !rhe_type.is_template() => {}
                _ => add_report(
                    ReportCode::WrongTypesInAssignOperation,
                    meta,
                    &mut analysis_information.reports,
                ),
            }
        }
        ConstraintEquality { lhe, rhe, .. } => {
            let lhe_response = type_expression(lhe, program_archive, analysis_information);
            let rhe_response = type_expression(rhe, program_archive, analysis_information);
            let lhe_type = if let Result::Ok(f) = lhe_response {
                f
            } else {
                return;
            };
            let rhe_type = if let Result::Ok(f) = rhe_response {
                f
            } else {
                return;
            };
            if lhe_type.is_template() {
                add_report(
                    ReportCode::MustBeArithmetic,
                    lhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            if rhe_type.is_template() {
                add_report(
                    ReportCode::MustBeArithmetic,
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            if rhe_type.dim() != lhe_type.dim() {
                add_report(
                    ReportCode::MustBeSameDimension,
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
        }
        LogCall { args, meta } => {
            for arglog in args {
                if let LogArgument::LogExp(arg) = arglog{
                    let arg_response = type_expression(arg, program_archive, analysis_information);
                    let arg_type = if let Result::Ok(t) = arg_response {
                        t
                    } else {
                        return;
                    };
                    if arg_type.is_template() || arg_type.dim() > 0 {
                        add_report(
                            ReportCode::MustBeSingleArithmetic,
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                }
            }
        }
        Assert { arg, meta } => {
            let arg_response = type_expression(arg, program_archive, analysis_information);
            let arg_type = if let Result::Ok(t) = arg_response {
                t
            } else {
                return;
            };
            if arg_type.is_template() || arg_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic,
                    meta,
                    &mut analysis_information.reports,
                )
            }
        }
        Return { value, meta } => {
            debug_assert!(analysis_information.return_type.is_some());
            let value_response = type_expression(value, program_archive, analysis_information);
            let value_type = if let Result::Ok(f) = value_response {
                f
            } else {
                return;
            };
            let ret_type = analysis_information.return_type.clone().unwrap();
            debug_assert!(!value_type.is_template());
            if ret_type != value_type.dim() {
                add_report(
                    ReportCode::ExpectedDimDiffGotDim(ret_type, value_type.dim()),
                    meta,
                    &mut analysis_information.reports,
                );
            }
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            let cond_response = type_expression(cond, program_archive, analysis_information);
            type_statement(if_case, program_archive, analysis_information);
            if let Option::Some(else_stmt) = else_case {
                type_statement(else_stmt, program_archive, analysis_information);
            }
            let cond_type = if let Result::Ok(t) = cond_response {
                t
            } else {
                return;
            };
            if cond_type.is_template() || cond_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            }
        }
        While { cond, stmt, .. } => {
            let cond_response = type_expression(cond, program_archive, analysis_information);
            type_statement(stmt, program_archive, analysis_information);
            let cond_type = if let Result::Ok(t) = cond_response {
                t
            } else {
                return;
            };
            if cond_type.is_template() || cond_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            }
        }
        Block { stmts, .. } => {
            analysis_information.environment.add_variable_block();
            for stmt in stmts.iter() {
                type_statement(stmt, program_archive, analysis_information)
            }
            analysis_information.environment.remove_variable_block();
        }
        MultSubstitution { .. } => unreachable!(),
    }
}
fn type_expression(
    expression: &Expression,
    program_archive: &ProgramArchive,
    analysis_information: &mut AnalysisInformation,
) -> Result<FoldedType, ()> {
    use Expression::*;
    match expression {
        Number(..) => Result::Ok(FoldedType::arithmetic_type(0)),
        ArrayInLine { meta, values } => {
            let values_types =
                type_array_of_expressions(values, program_archive, analysis_information)?;
            if values_types.is_empty() {
                return add_report_and_end(
                    ReportCode::EmptyArrayInlineDeclaration,
                    meta,
                    &mut analysis_information.reports,
                );
            }
            let inferred_dim = values_types[0].dim();
            for (expression, value_type) in values.iter().zip(values_types.iter()) {
                if value_type.is_template() {
                    add_report(
                        ReportCode::InvalidArrayType,
                        expression.get_meta(),
                        &mut analysis_information.reports,
                    );
                } else if inferred_dim != value_type.dim() {
                    add_report(
                        ReportCode::NonHomogeneousArray,
                        expression.get_meta(),
                        &mut analysis_information.reports,
                    );
                }
            }
            Result::Ok(FoldedType::arithmetic_type(inferred_dim + 1))
        }
        UniformArray { meta, value, dimension } => {
            let value_type = type_expression(value, program_archive, analysis_information).unwrap();
            if value_type.is_template() {
                add_report(
                    ReportCode::InvalidArrayType,
                    meta,
                    &mut analysis_information.reports,
                );
            };
            let dim_type = type_expression(dimension, program_archive, analysis_information).unwrap();
            if dim_type.is_template() {
                add_report(
                    ReportCode::InvalidArrayType,
                    expression.get_meta(),
                    &mut analysis_information.reports,
                );
            } else if dim_type.dim() != 0 {
                add_report(
                    ReportCode::InvalidArrayType,
                    expression.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            
            Result::Ok(FoldedType::arithmetic_type(value_type.dim() + 1))
        }
        InfixOp { lhe, rhe, .. } => {
            let lhe_response = type_expression(lhe, program_archive, analysis_information);
            let rhe_response = type_expression(rhe, program_archive, analysis_information);
            let lhe_type = lhe_response?;
            let rhe_type = rhe_response?;
            let mut successful = Result::Ok(());
            if lhe_type.is_template() || lhe_type.dim() > 0 {
                successful = add_report_and_end(
                    ReportCode::InfixOperatorWithWrongTypes,
                    lhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            if rhe_type.is_template() || rhe_type.dim() > 0 {
                successful = add_report_and_end(
                    ReportCode::InfixOperatorWithWrongTypes,
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            successful?;
            Result::Ok(FoldedType::arithmetic_type(0))
        }
        PrefixOp { rhe, .. } => {
            let rhe_type = type_expression(rhe, program_archive, analysis_information)?;
            if rhe_type.is_template() || rhe_type.dim() > 0 {
                add_report_and_end(
                    ReportCode::PrefixOperatorWithWrongTypes,
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                )
            } else {
                Result::Ok(FoldedType::arithmetic_type(0))
            }
        }
        ParallelOp {rhe, .. } =>{
            let rhe_type = type_expression(rhe, program_archive, analysis_information)?;
            if rhe_type.is_template()  {
                Result::Ok(rhe_type)
            } else {
                add_report_and_end(
                    ReportCode::ParallelOperatorWithWrongTypes,
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                )
            }
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            let cond_response = type_expression(cond, program_archive, analysis_information);
            let if_true_response = type_expression(if_true, program_archive, analysis_information);
            let if_false_response =
                type_expression(if_false, program_archive, analysis_information);
            let if_true_type = if_true_response?;

            let cond_type = if let Result::Ok(f) = cond_response {
                f
            } else {
                return Result::Ok(if_true_type);
            };
            if cond_type.is_template() || cond_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                );
            }

            let if_false_type = if let Result::Ok(f) = if_false_response {
                f
            } else {
                return Result::Ok(if_true_type);
            };
            if !FoldedType::same_type(&if_true_type, &if_false_type) {
                add_report(
                    ReportCode::NonCompatibleBranchTypes,
                    if_false.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            Result::Ok(if_true_type)
        }
        Variable { name, access, meta, .. } => {
            debug_assert!(analysis_information.environment.has_symbol(name));
            let access_information =
                treat_access(name, access, meta, program_archive, analysis_information)?;
            let environment = &analysis_information.environment;
            let reports = &mut analysis_information.reports;
            let symbol_information = apply_access_to_symbol(
                name,
                meta,
                access_information,
                environment,
                reports,
                program_archive,
            )?;
            match symbol_information {
                SymbolInformation::Component(possible_template) if possible_template.is_some() => {
                    Result::Ok(FoldedType::template(&possible_template.unwrap()))
                }
                SymbolInformation::Var(dim) | SymbolInformation::Signal(dim) => {
                    Result::Ok(FoldedType::arithmetic_type(dim))
                }
                SymbolInformation::Tag => {
                    Result::Ok(FoldedType::arithmetic_type(0))
                }
                SymbolInformation::Component(possible_template) if possible_template.is_none() => {
                    add_report_and_end(ReportCode::UninitializedSymbolInExpression, meta, reports)
                }
                _ => unreachable!(),
            }
        }
        Call { id, args, meta } => {
            analysis_information.reached.insert(id.clone());
            let typing_response =
                type_array_of_expressions(args, program_archive, analysis_information);
            if program_archive.contains_template(id) && typing_response.is_err() {
                return Result::Ok(FoldedType::template(id));
            }
            let arg_types = typing_response?;
            let mut concrete_types = Vec::new();
            let mut success = Result::Ok(());
            for (arg_expr, arg_type) in args.iter().zip(arg_types.iter()) {
                if arg_type.is_template() {
                    success = add_report_and_end(
                        ReportCode::InvalidArgumentInCall,
                        arg_expr.get_meta(),
                        &mut analysis_information.reports,
                    );
                }
                concrete_types.push(arg_type.dim());
            }
            if program_archive.contains_template(id) && success.is_err() {
                return Result::Ok(FoldedType::template(id));
            }
            success?;
            let previous_file_id = analysis_information.file_id;
            analysis_information.file_id = if program_archive.contains_function(id) {
                program_archive.get_function_data(id).get_file_id()
            } else {
                program_archive.get_template_data(id).get_file_id()
            };
            let new_environment = prepare_environment_for_call(
                meta,
                id,
                &concrete_types,
                program_archive,
                &mut analysis_information.reports,
            );
            if new_environment.is_err() {
                return Result::Ok(FoldedType::template(id));
            }
            let new_environment = new_environment?;
            let previous_environment =
                std::mem::replace(&mut analysis_information.environment, new_environment);
            let returned_type = if program_archive.contains_function(id) {
                type_function(id, &concrete_types, meta, analysis_information, program_archive)
                    .map(|val| FoldedType::arithmetic_type(val))
            } else {
                let r_val =
                    type_template(id, &concrete_types, analysis_information, program_archive);
                Result::Ok(FoldedType::template(&r_val))
            };
            analysis_information.environment = previous_environment;
            analysis_information.file_id = previous_file_id;
            let folded_value = returned_type?;
            Result::Ok(folded_value)
        }
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}
//************************************************* Statement support *************************************************
fn treat_sequence_of_statements(
    stmts: &[Statement],
    program_archive: &ProgramArchive,
    analysis_information: &mut AnalysisInformation,
) {
    for stmt in stmts.iter() {
        type_statement(stmt, program_archive, analysis_information);
    }
}

//************************************************* Expression support *************************************************
// 0: symbol dimensions accessed
// 1: Signal accessed and dimensions accessed in that signal (optional)
type AccessInfo = (ArithmeticType, Option<(String, ArithmeticType)>, Option<String>);
fn treat_access(
    var: &String,
    accesses: &[Access],
    meta: &Meta,
    program_archive: &ProgramArchive,
    analysis_information: &mut AnalysisInformation,
) -> Result<AccessInfo, ()> {
    use Access::*;
    let mut access_info: AccessInfo = (0, Option::None, Option::None);
    let mut successful = Result::Ok(());
    for access in accesses {
        match access {
            ArrayAccess(index) => {
                let index_response = type_expression(index, program_archive, analysis_information);
                if let Option::Some(signal_info) = &mut access_info.1 {
                    signal_info.1 += 1;
                } else {
                    access_info.0 += 1;
                }
                if let Result::Ok(index_type) = index_response {
                    if index_type.is_template() || index_type.dim() > 0 {
                        add_report(
                            ReportCode::InvalidArraySize,
                            index.get_meta(),
                            &mut analysis_information.reports,
                        );
                    }
                }
            }
            ComponentAccess(name) => {
                if let Option::Some(signal_info) = & access_info.1 {
                    let accessed_comp = analysis_information.environment.get_component(var).unwrap().0.as_ref().unwrap();  
                    let comp_info = program_archive.get_template_data(accessed_comp);
                    let comp_outputs = comp_info.get_outputs();
                    let comp_inputs = comp_info.get_inputs();
                    if signal_info.1 > 0 {
                        add_report(
                            ReportCode::InvalidArraySize,
                            meta,
                            &mut analysis_information.reports,
                        );
                    }
                    else if comp_inputs.contains_key(&signal_info.0) {
                        successful = add_report_and_end(
                            ReportCode::InvalidSignalTagAccess, //We can report more exactly input signals cannot be accessed.
                            meta,
                            &mut analysis_information.reports,
                        );
                    } else if comp_outputs.contains_key(&signal_info.0) {
                        let output_info = &comp_outputs.get(&signal_info.0).unwrap().1;
                        if !output_info.contains(name) || access_info.2.is_some() {
                            successful = add_report_and_end(
                                ReportCode::InvalidSignalTagAccess,
                                  meta,
                                  &mut analysis_information.reports,
                                );
                        } else {
                            access_info.2 = Option::Some(name.clone());
                        }
                    }
                    else {
                      successful = add_report_and_end(
                        ReportCode::InvalidSignalTagAccess,
                          meta,
                          &mut analysis_information.reports,
                        );
                    }
                } else {
                    access_info.1 = Option::Some((name.clone(), 0));
                }
            }
        }
    }
    successful?;
    Result::Ok(access_info)
}

enum SymbolInformation {
    Component(Option<String>),
    Var(ArithmeticType),
    Signal(ArithmeticType),
    Tag,
}
fn apply_access_to_symbol(
    symbol: &str,
    meta: &Meta,
    access_information: AccessInfo,
    environment: &TypingEnvironment,
    reports: &mut ReportCollection,
    program_archive: &ProgramArchive,
) -> Result<SymbolInformation, ()> {
    let (current_template, mut current_dim, possible_tags) = if environment.has_component(symbol) {
        let (temp, dim) = environment.get_component_or_break(symbol, file!(), line!()).clone();
        (temp,dim, Vec::new())
    } else if environment.has_signal(symbol) {
        let(dim, tags) = environment.get_signal_or_break(symbol, file!(), line!());
        (Option::None,  *dim, tags.clone())
    } else {
        let dim = environment.get_variable_or_break(symbol, file!(), line!());
        (Option::None, *dim, Vec::new())
    };

    if access_information.0 > current_dim {
        return add_report_and_end(ReportCode::InvalidArrayAccess, meta, reports);
    } else {
        current_dim -= access_information.0
    }

    if access_information.0 == 0 && environment.has_component(symbol) && access_information.1.is_some() && access_information.2.is_some() {
            Result::Ok(SymbolInformation::Tag)
    }
    else if access_information.1.is_some() && environment.has_signal(symbol){
         if access_information.0 == 0 && contains_the_tag(access_information.1.clone(), &possible_tags)
        {
            Result::Ok(SymbolInformation::Tag)
        }
        else {
            if access_information.0 == 0 {
                add_report_and_end(ReportCode::InvalidTagAccess, meta, reports)
            }
            else {
                add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports)
            }
        }
    }
    else if access_information.1.is_some() && (current_dim > 0 || current_template.is_none()) {
            add_report_and_end(ReportCode::InvalidSignalAccess, meta, reports)
    } else if let Option::Some((signal_name, dims_accessed)) = access_information.1 {
        let template_name = current_template.unwrap();
        let input = program_archive.get_template_data(&template_name).get_input_info(&signal_name);
        let output =
            program_archive.get_template_data(&template_name).get_output_info(&signal_name);
        current_dim = match (input, output) {
            (Option::Some((d, _)), _) | (_, Option::Some((d, _))) => *d,
            _ => {
                return add_report_and_end(ReportCode::InvalidSignalAccess, meta, reports);
            }
        };
        if dims_accessed > current_dim {
            add_report_and_end(ReportCode::InvalidArrayAccess, meta, reports)
        } else {
            Result::Ok(SymbolInformation::Signal(current_dim - dims_accessed))
        }
    } else if environment.has_variable(symbol) {
        Result::Ok(SymbolInformation::Var(current_dim))
    } else if environment.has_signal(symbol) {
        Result::Ok(SymbolInformation::Signal(current_dim))
    } else if environment.has_component(symbol) && current_dim == 0 {
        Result::Ok(SymbolInformation::Component(current_template))
    } else {
        add_report_and_end(ReportCode::InvalidPartialArray, meta, reports)
    }
}

fn contains_the_tag(access_information: Option<(String, usize)>, tags: &Vec<String>) -> bool {
    if let Option::Some(access) = access_information {
            tags.contains(&access.0)
    }
    else {false}
}

fn type_array_of_expressions(
    expressions: &[Expression],
    program_archive: &ProgramArchive,
    analysis_information: &mut AnalysisInformation,
) -> Result<Vec<FoldedType>, ()> {
    let mut types = Vec::new();
    let mut successful_typing = true;
    for expression in expressions {
        let typing_result = type_expression(expression, program_archive, analysis_information);
        if let Result::Ok(expression_type) = typing_result {
            types.push(expression_type);
        } else {
            successful_typing = false;
        }
    }
    if successful_typing {
        Result::Ok(types)
    } else {
        Result::Err(())
    }
}

fn prepare_environment_for_call(
    meta: &Meta,
    call_id: &str,
    args_dims: &[ArithmeticType],
    program_archive: &ProgramArchive,
    reports: &mut ReportCollection,
) -> Result<TypingEnvironment, ()> {
    let args_names = if program_archive.contains_function(call_id) {
        program_archive.get_function_data(call_id).get_name_of_params()
    } else {
        program_archive.get_template_data(call_id).get_name_of_params()
    };
    if args_dims.len() != args_names.len() {
        let error_code = ReportCode::WrongNumberOfArguments(args_names.len(), args_dims.len());
        add_report_and_end(error_code, meta, reports)?;
    }
    let mut environment = TypingEnvironment::new();
    for (name, dim) in args_names.iter().zip(args_dims.iter()) {
        environment.add_variable(name, *dim);
    }
    Result::Ok(environment)
}

fn type_template(
    call_id: &str,
    args_dims: &[ArithmeticType],
    analysis_information: &mut AnalysisInformation,
    program_archive: &ProgramArchive,
) -> String {
    debug_assert!(program_archive.contains_template(call_id));
    if analysis_information.registered_calls.get_instance(call_id, args_dims).is_none() {
        analysis_information.registered_calls.add_instance(call_id, args_dims.to_vec(), 0);
        let stmts = program_archive.get_template_data(call_id).get_body_as_vec();
        treat_sequence_of_statements(stmts, program_archive, analysis_information);
    }
    call_id.to_string()
}

fn type_function(
    call_id: &str,
    args_dims: &[ArithmeticType],
    meta: &Meta,
    analysis_information: &mut AnalysisInformation,
    program_archive: &ProgramArchive,
) -> Result<ArithmeticType, ()> {
    debug_assert!(program_archive.contains_function(call_id));
    if let Option::Some(instance) =
        analysis_information.registered_calls.get_instance(call_id, args_dims)
    {
        return Result::Ok(*instance.returns());
    }
    let mut given_type = type_given_function(call_id, program_archive.get_functions(), args_dims);
    if let Option::Some(raw) = &given_type {
        analysis_information.registered_calls.add_instance(call_id, args_dims.to_vec(), *raw);
    } else {
        return add_report_and_end(
            ReportCode::UnableToTypeFunction,
            meta,
            &mut analysis_information.reports,
        );
    }
    let stmts = program_archive.get_function_data(call_id).get_body_as_vec();
    let previous_type = std::mem::replace(&mut analysis_information.return_type, given_type);
    treat_sequence_of_statements(stmts, program_archive, analysis_information);
    given_type = std::mem::replace(&mut analysis_information.return_type, previous_type);
    debug_assert!(given_type.is_some());
    let raw_type = given_type.unwrap();
    Result::Ok(raw_type)
}

//************************************************* Report handling *************************************************
fn add_report_and_end<Ok>(
    error_code: ReportCode,
    meta: &Meta,
    reports: &mut ReportCollection,
) -> Result<Ok, ()> {
    add_report(error_code, meta, reports);
    Result::Err(())
}

fn add_report(error_code: ReportCode, meta: &Meta, reports: &mut ReportCollection) {
    use ReportCode::*;
    let file_id = meta.get_file_id();
    let mut report = Report::error("Typing error found".to_string(), error_code);
    let location = generate_file_location(meta.start, meta.end);
    let message = match error_code {
        TypeCantBeUseAsCondition => "This type can not be used as a condition".to_string(),
        BadArrayAccess => "This type can not be used as index".to_string(),
        EmptyArrayInlineDeclaration => "Empty arrays can not be declared inline".to_string(),
        NonHomogeneousArray => "All the elements in a array must have the same type".to_string(),
        InvalidArraySize => {
            "Array indexes and lengths must be single arithmetic expressions".to_string()
        }
        InvalidArrayAccess => {
            "Array access does not match the dimensions of the expression".to_string()
        }
        InvalidSignalAccess => "Signal not found in component".to_string(),
        InvalidSignalTagAccess => "The latest access cannot be done from component".to_string(),
        InvalidTagAccess => "Tag not found in signal".to_string(),
        InvalidTagAccessAfterArray => "Tag cannot be found after an array access".to_string(),
        InvalidArrayType => "Components can not be declared inside inline arrays".to_string(),
        InfixOperatorWithWrongTypes | PrefixOperatorWithWrongTypes => {
            "Type not allowed by the operator".to_string()
        }
        ParallelOperatorWithWrongTypes  => {
            "Type not allowed by the operator parallel (needs a template)".to_string()
        }
        InvalidPartialArray => "Only variable arrays can be accessed partially".to_string(),
        UninitializedSymbolInExpression => "The type of this symbol is not known".to_string(),
        WrongTypesInAssignOperation => "Assignee and assigned types do not match".to_string(),
        InvalidArgumentInCall => "Components can not be passed as arguments".to_string(),
        UnableToTypeFunction => "Unable to infer the type of this function".to_string(),
        MustBeSingleArithmetic => "Must be a single arithmetic expression".to_string(),
        MustBeArithmetic => "Must be a single arithmetic expression or an array".to_string(),
        OutputTagCannotBeModifiedOutside => "Output tag from a subcomponent cannot be modified".to_string(),
        MustBeSameDimension => "Must be two arrays of the same dimensions".to_string(),
        MainComponentWithTags => "Main component cannot have inputs with tags".to_string(),
        ExpectedDimDiffGotDim(expected, got) => {
            format!("Function should return {} but returns {}", expected, got)
        }
        WrongNumberOfArguments(expected, got) => {
            format!("Expecting {} arguments, {} where obtained", expected, got)
        }
        _ => panic!("Unimplemented error code"),
    };
    report.add_primary(location, file_id, message);
    reports.push(report);
}
