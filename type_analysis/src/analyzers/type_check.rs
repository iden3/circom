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
            ReportCode::WrongTypesInAssignOperationTemplate,
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
                if dim_type.is_template() {
                    add_report(
                        ReportCode::InvalidArraySizeT,
                        dim_expression.get_meta(),
                        &mut analysis_information.reports,
                    );
                }
                else if dim_type.dim() > 0 {
                    add_report(
                        ReportCode::InvalidArraySize(dim_type.dim()),
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
                treat_access(access, meta, program_archive, analysis_information);

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
                (SymbolInformation::Signal(_), AssignOp::AssignVar)=>{
                    return add_report(
                        ReportCode::WrongTypesInAssignOperationOperatorSignal,
                        meta,
                        &mut analysis_information.reports,
                    );
                }
                _ => {
                    return add_report(
                        ReportCode::WrongTypesInAssignOperationOperatorNoSignal,
                        meta,
                        &mut analysis_information.reports,
                    );
                }
            }
            match symbol_information {
                SymbolInformation::Component(possible_template) =>{
                    if rhe_type.is_template(){
                        if possible_template.is_none(){
                            let (current_template, _) = analysis_information
                                .environment
                                .get_mut_component_or_break(var, file!(), line!());
                            *current_template = rhe_type.template;
                        } else{
                            let template = possible_template.unwrap();
                            let r_template = rhe_type.template.unwrap();
                            if template != r_template {
                                add_report(
                                    ReportCode::WrongTypesInAssignOperationArrayTemplates,
                                    meta,
                                    &mut analysis_information.reports,
                                )
                            }
                        }
                    } else{
                        add_report(
                            ReportCode::WrongTypesInAssignOperationTemplate,
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                }
                SymbolInformation::Signal(dim) =>{
                    if rhe_type.is_template(){
                        add_report(
                            ReportCode::WrongTypesInAssignOperationExpression,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if dim != rhe_type.dim(){
                        add_report(
                            ReportCode::WrongTypesInAssignOperationDims(dim, rhe_type.dim()),
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                    
                }
                SymbolInformation::Var(dim) =>{
                    if rhe_type.is_template(){
                        add_report(
                            ReportCode::WrongTypesInAssignOperationExpression,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if dim != rhe_type.dim(){
                        add_report(
                            ReportCode::WrongTypesInAssignOperationDims(dim, rhe_type.dim()), 
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                    
                }
                SymbolInformation::Tag =>{
                    if rhe_type.is_template(){
                        add_report(
                            ReportCode::WrongTypesInAssignOperationExpression,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if 0 != rhe_type.dim(){
                        add_report(
                            ReportCode::WrongTypesInAssignOperationDims(0, rhe_type.dim()),
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                    
                }
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
                    ReportCode::MustBeSameDimension(rhe_type.dim(), lhe_type.dim()),
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
        }
        LogCall { args, meta } => {
            for arglog in args {
                if let LogArgument::LogExp(arg) = arglog{
                    let arg_response = type_expression(&arg, program_archive, analysis_information);
                    let arg_type = if let Result::Ok(t) = arg_response {
                        t
                    } else {
                        return;
                    };
                    if arg_type.is_template()  {
                        add_report(
                            ReportCode::MustBeSingleArithmeticT,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if arg_type.dim() > 0 {
                        add_report(
                            ReportCode::MustBeSingleArithmetic(arg_type.dim()),
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
            if arg_type.is_template() {
                add_report(
                    ReportCode::MustBeSingleArithmeticT,
                    meta,
                    &mut analysis_information.reports,
                )
            } else if arg_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic(arg_type.dim()),
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
            if cond_type.is_template(){
                add_report(
                    ReportCode::MustBeSingleArithmeticT,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            }else if cond_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic(cond_type.dim()),
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
            if cond_type.is_template(){
                add_report(
                    ReportCode::MustBeSingleArithmeticT,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            }else if cond_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic(cond_type.dim()),
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
        UnderscoreSubstitution { rhe , ..} => {
            let rhe_response = type_expression(rhe, program_archive, analysis_information);
            let rhe_type = if let Result::Ok(r_type) = rhe_response {
                r_type
            } else {
                return;
            };
            if rhe_type.is_template() {
                add_report(
                    ReportCode::MustBeArithmetic,
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
        },
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
                        ReportCode::NonHomogeneousArray(inferred_dim, value_type.dim()),
                        expression.get_meta(),
                        &mut analysis_information.reports,
                    );
                }
            }
            Result::Ok(FoldedType::arithmetic_type(inferred_dim + 1))
        }
        UniformArray { meta, value, dimension } => {
            let value_type = type_expression(value, program_archive, analysis_information)?;
            if value_type.is_template() {
                add_report(
                    ReportCode::InvalidArrayType,
                    meta,
                    &mut analysis_information.reports,
                );
            };
            let dim_type = type_expression(dimension, program_archive, analysis_information)?;
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
            if cond_type.is_template(){
                add_report(
                    ReportCode::MustBeSingleArithmeticT,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            }
            else if cond_type.dim() > 0 {
                add_report(
                    ReportCode::MustBeSingleArithmetic(cond_type.dim()),
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
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
                treat_access( access, meta, program_archive, analysis_information)?;
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
// 2: Tag accessed (optional)
type AccessInfo = (ArithmeticType, Option<(String, ArithmeticType)>, Option<String>);
fn treat_access(
    accesses: &[Access],
    meta: &Meta,
    program_archive: &ProgramArchive,
    analysis_information: &mut AnalysisInformation,
) -> Result<AccessInfo, ()> {
    use Access::*;
    let mut access_info: AccessInfo = (0, Option::None, Option::None);
    for access in accesses {
        match access {
            ArrayAccess(index) => {
                let index_response = type_expression(&index, program_archive, analysis_information);
                
                if access_info.2.is_some(){
                    add_report(
                        ReportCode::InvalidArrayAccess(0, 1),
                        index.get_meta(),
                        &mut analysis_information.reports,
                    );
                } else{
                    if let Option::Some(signal_info) = &mut access_info.1 {
                        signal_info.1 += 1;
                    } else {
                        access_info.0 += 1;
                    }
                    if let Result::Ok(index_type) = index_response {
                        if index_type.is_template() {
                            add_report(
                                ReportCode::InvalidArraySizeT,
                                index.get_meta(),
                                &mut analysis_information.reports,
                            );
                        }
                        else if index_type.dim() > 0 {
                            add_report(
                                ReportCode::InvalidArraySize(index_type.dim()),
                                index.get_meta(),
                                &mut analysis_information.reports,
                            );
                        }
                    }
                }
            }
            ComponentAccess(name) => {
                if let Option::Some(_signal_info) = & access_info.1 {
                    if access_info.2.is_none(){
                        access_info.2 = Some(name.clone())
                    } else{
                        add_report(
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
        return add_report_and_end(ReportCode::InvalidArrayAccess(current_dim, access_information.0), meta, reports);
    } else {
        current_dim -= access_information.0
    }

    // Case signals or tags 
    if let Option::Some((signal_name, dims_accessed)) = access_information.1{
        if current_template.is_some(){ // we are inside component
            
            if current_dim != 0{ // only allowed complete accesses to component
                return add_report_and_end(ReportCode::InvalidPartialArray, meta, reports);
            }

            let template_name = current_template.unwrap();
            let input = program_archive.get_template_data(&template_name).get_input_info(&signal_name);
            let output = program_archive.get_template_data(&template_name).get_output_info(&signal_name);
            let tags;
            (current_dim, tags) = match (input, output) {
                (Option::Some((d, tags)), _) | (_, Option::Some((d, tags))) => (*d, tags),
                _ => {
                    return add_report_and_end(ReportCode::InvalidSignalAccess, meta, reports);
                }
            };
            if access_information.2.is_some(){ // tag of io signal of component
                if dims_accessed > 0{
                    return add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports);
                }
                else if !tags.contains(&access_information.2.unwrap()){
                    return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                } else{
                    return Result::Ok(SymbolInformation::Tag);
                }
            } else{ // io signal of component
                if dims_accessed > current_dim {
                    return add_report_and_end(ReportCode::InvalidArrayAccess(current_dim, dims_accessed), meta, reports);
                } else {
                    return Result::Ok(SymbolInformation::Signal(current_dim - dims_accessed));
                }   
            }
        } else{ // we are in template
            if environment.has_signal(symbol){
                if access_information.0 != 0{
                    add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports)
                } else if dims_accessed > 0{
                    add_report_and_end(
                        ReportCode::InvalidArrayAccess(0, dims_accessed),
                        meta,
                        reports,
                    )
                } else if !possible_tags.contains(&signal_name){
                    add_report_and_end(ReportCode::InvalidTagAccess, meta, reports)
                } else{
                    Result::Ok(SymbolInformation::Tag)
                }
            
            } else if environment.has_component(symbol){
                add_report_and_end(ReportCode::UninitializedComponent, meta, reports)
            } else{
                add_report_and_end(ReportCode::InvalidSignalTagAccess, meta, reports)
            }
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
        //TypeCantBeUseAsCondition => "This type can not be used as a condition".to_string(),
        //BadArrayAccess => "This type can not be used as index".to_string(),
        EmptyArrayInlineDeclaration => "Empty arrays can not be declared inline".to_string(),
        NonHomogeneousArray(dim_1, dim_2) => 
            format!("All the elements in a array must have the same type.\n Found elements in the array with {} and {} dimensions.",
                dim_1, dim_2),
        InvalidArraySize(dim) => {
            format!("Array indexes and lengths must be single arithmetic expressions.\n Found expression with {} dimensions.",
                dim)
        }
        InvalidArraySizeT =>{
            "Array indexes and lengths must be single arithmetic expressions.\n Found component instead of expression.".to_string()
        }
        InvalidArrayAccess(expected, given) => {
            format!("Array access does not match the dimensions of the expression. \n Expected {} dimensions, given {}.",
                expected, given
            )
        }
        InvalidSignalAccess => "Signal not found in component: only accesses to input/output signals are allowed".to_string(),
        InvalidSignalTagAccess => "Invalid tag access: could not find the tag".to_string(),
        InvalidTagAccess => "Tag not found in signal: only accesses to tags that appear in the definition of the signal are allowed".to_string(),
        InvalidTagAccessAfterArray => "Invalid access to the tag of an array element: tags belong to complete arrays, not to individual positions.\n Hint: instead of signal[pos].tag use signal.tag".to_string(),
        InvalidArrayType => "Components can not be declared inside inline arrays".to_string(),
        InfixOperatorWithWrongTypes | PrefixOperatorWithWrongTypes => {
            "Type not allowed by the operator".to_string()
        }
        ParallelOperatorWithWrongTypes  => {
            "Type not allowed by the operator parallel (parallel operator can only be applied to templates)".to_string()
        }
        InvalidPartialArray => "Only variable arrays can be accessed partially".to_string(),
        UninitializedSymbolInExpression => "The type of this symbol is not known".to_string(),
        WrongTypesInAssignOperationOperatorSignal => {
            format!("The operator does not match the types of the assigned elements.\n Assignments to signals do not allow the operator =, try using <== or <-- instead")
        }
        WrongTypesInAssignOperationOperatorNoSignal => {
            format!("The operator does not match the types of the assigned elements.\n Only assignments to signals allow the operators <== and <--, try using = instead")
        }
        WrongTypesInAssignOperationArrayTemplates => "Assignee and assigned types do not match.\n All components of an array must be instances of the same template.".to_string(),
        WrongTypesInAssignOperationTemplate => "Assignee and assigned types do not match.\n Expected template found expression.".to_string(),
        WrongTypesInAssignOperationExpression => "Assignee and assigned types do not match.\n Expected expression found template.".to_string(),
        WrongTypesInAssignOperationDims(expected, found) => {
            format!("Assignee and assigned types do not match. \n Expected dimensions: {}, found {}",
            expected, found)
        }
        InvalidArgumentInCall => "Components can not be passed as arguments".to_string(),
        UnableToTypeFunction => "Unable to infer the type of this function".to_string(),
        MustBeSingleArithmetic(dim) => {
            format!("Must be a single arithmetic expression.\n Found expression of {} dimensions", dim)
        }
        MustBeSingleArithmeticT => {
              format!("Must be a single arithmetic expression.\n Found component")
        }
        MustBeArithmetic => "Must be a single arithmetic expression or an array of arithmetic expressions. \n Found component".to_string(),
        OutputTagCannotBeModifiedOutside => "Output tag from a subcomponent cannot be modified".to_string(),
        MustBeSameDimension(dim_1, dim_2) =>{
            format!("Must be two arrays of the same dimensions.\n Found {} and {} dimensions", dim_1, dim_2)
        }
        MainComponentWithTags => "Main component cannot have inputs with tags".to_string(),
        ExpectedDimDiffGotDim(expected, got) => {
            format!("Function should return {} but returns {}", expected, got)
        }
        WrongNumberOfArguments(expected, got) => {
            format!("Expecting {} arguments, {} where obtained", expected, got)
        }
        UninitializedComponent => "Trying to access to a signal of a component that has not been initialized".to_string(),
        NonCompatibleBranchTypes => "Inline switch operator branches types are non compatible".to_string(),
        e => panic!("Unimplemented error code: {}", e),
    };
    report.add_primary(location, file_id, message);
    reports.push(report);
}
