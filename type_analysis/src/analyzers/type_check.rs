use super::type_given_function::type_given_function;
use super::type_register::TypeRegister;
use program_structure::ast::*;
use program_structure::ast::Expression::Call;
use program_structure::environment::CircomEnvironment;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::{generate_file_location, FileID};
use program_structure::program_archive::ProgramArchive;
use program_structure::wire_data::WireType;
use std::collections::HashSet;


type ArithmeticType = usize;
type ComponentInfo = (Option<String>, ArithmeticType);
type BusInfo = (Option<String>, ArithmeticType, std::vec::Vec<std::string::String>);
type SignalInfo = (ArithmeticType, std::vec::Vec<std::string::String>);
type VarInfo = ArithmeticType;
type TypingEnvironment = CircomEnvironment<ComponentInfo, SignalInfo, VarInfo, BusInfo>;
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
    // var dimension
    arithmetic: Option<ArithmeticType>,
    // template name
    template: Option<String>,
    // bus type name
    bus: Option<String>,
}
impl FoldedType {
    pub fn arithmetic_type(dimensions: ArithmeticType) -> FoldedType {
        FoldedType { arithmetic: Option::Some(dimensions), template: Option::None, bus: Option::None }
    }
    pub fn template(name: &str) -> FoldedType {
        FoldedType { template: Option::Some(name.to_string()), arithmetic: Option::None, bus: Option::None }
    }
    pub fn is_template(&self) -> bool {
        self.template.is_some() && self.arithmetic.is_none()
    }
    pub fn bus(name: &str, dimensions: ArithmeticType) -> FoldedType {
        FoldedType { bus: Option::Some(name.to_string()), arithmetic: Option::Some(dimensions), template: Option::None }
    }
    pub fn is_bus(&self) -> bool {
        self.bus.is_some()
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
        if let (Option::Some(l_template), Option::Some(r_template)) = (&left.template, &right.template) {
            equal = l_template.eq(r_template);
        }
        if let (Option::Some(l_bus), Option::Some(r_bus)) = (&left.bus, &right.bus) {
            equal = l_bus.eq(r_bus);
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
    if type_analysis_response.is_err(){
        return Result::Err(analysis_information.reports);
    }

    check_main_has_tags(initial_expression, program_archive, &mut analysis_information.reports);


    if analysis_information.reports.is_empty() {
        Result::Ok(OutInfo { reached: analysis_information.reached })
    } else {
        Result::Err(analysis_information.reports)
    }
}

fn check_main_has_tags(initial_expression: &Expression, program_archive: &ProgramArchive, reports: &mut ReportCollection) {    if let Call { id, .. } = initial_expression {
        if program_archive.contains_template(id){
            let inputs = program_archive.get_template_data(id).get_inputs();
            for (_name,info) in inputs {
                if !info.get_tags().is_empty(){
                    add_report(
                        ReportCode::MainComponentWithTags,
                        initial_expression.get_meta(),
                        reports,
                    );
                    break;
                } else if let WireType::Bus(bus_name) = info.get_type() {
                    if check_bus_contains_tag_recursive(bus_name, program_archive){
                        add_report(
                            ReportCode::MainComponentWithTags,
                            initial_expression.get_meta(),
                            reports,
                        );
                        break;
                    }
                }
            }
        } else{
            add_report(
                ReportCode::IllegalMainExpression,
                initial_expression.get_meta(),
                reports,
            )
        }
       
    }
    else { 
        add_report(
            ReportCode::IllegalMainExpression,
            initial_expression.get_meta(),
            reports,
        )
    }
}

fn check_bus_contains_tag_recursive(bus_name: String, program_archive: &ProgramArchive) -> bool {
    let bus_data = program_archive.get_bus_data(&bus_name);
    let mut tag_in_inputs = false;
    for (_name, info) in bus_data.get_fields() {
        if !info.get_tags().is_empty(){
            tag_in_inputs = true;
            break;
        } else if let WireType::Bus(bus_name) = info.get_type() {
            if check_bus_contains_tag_recursive(bus_name, program_archive){
                tag_in_inputs = true;
                break;
            }
        }
    }
    tag_in_inputs
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
                else if dim_type.is_bus() {
                    add_report(
                        ReportCode::InvalidArraySizeB,
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
                    match s_type {
                        SignalType::Input => analysis_information
                            .environment
                            .add_input(name, (dimensions.len(),tags.clone())),
                        SignalType::Output => analysis_information
                            .environment
                            .add_output(name, (dimensions.len(),tags.clone())),
                        SignalType::Intermediate => analysis_information
                            .environment
                            .add_intermediate(name, (dimensions.len(),tags.clone())),
                    }
                }
                VariableType::Var => analysis_information
                    .environment
                    .add_variable(name, dimensions.len()),
                VariableType::Component => analysis_information
                    .environment
                    .add_component(name, (meta.component_inference.clone(), dimensions.len())),
                VariableType::AnonymousComponent => analysis_information
                    .environment
                    .add_component(name, (meta.component_inference.clone(), dimensions.len())),
                VariableType::Bus(tname, ss_type, tags) => {
                    match ss_type {
                        SignalType::Input => analysis_information
                            .environment
                            .add_input_bus(name, (Option::Some(tname.clone()), dimensions.len(), tags.clone())),
                        SignalType::Output => analysis_information
                            .environment
                            .add_output_bus(name, (Option::Some(tname.clone()), dimensions.len(), tags.clone())),
                        SignalType::Intermediate => analysis_information
                            .environment
                            .add_intermediate_bus(name, (Option::Some(tname.clone()), dimensions.len(), tags.clone())),
                    }
                }
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
                treat_access(access, program_archive, analysis_information);

            let access_information = if let Result::Ok(info) = access_information_result {
                info
            } else {
                return;
            };

            let symbol_type_result = apply_access_to_symbol(
                var,
                meta,
                access_information.clone(),
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
                | (SymbolInformation::Component(_), AssignOp::AssignVar)
                | (SymbolInformation::Bus(_,_), AssignOp::AssignConstraintSignal)
                | (SymbolInformation::Bus(_,_), AssignOp::AssignSignal)  => {}
                | (SymbolInformation::Bus(_,_), AssignOp::AssignVar) => {
                    if !rhe.is_bus_call() && !rhe.is_bus_call_array(){
                        return add_report(ReportCode::WrongTypesInAssignOperationBus, meta, &mut analysis_information.reports);
                    }
                }
                | (SymbolInformation::Tag, AssignOp::AssignVar) => {
                    //If the tag comes from an output wire, it cannot be modified.
                    if analysis_information.environment.has_component(var){
                        let (comp,_) = analysis_information.environment.get_component(var).unwrap();
                        if comp.is_some() && access_information.1.is_some() && access_information.1.as_ref().unwrap().len() > 0 {
                            let template_name = comp.clone().unwrap();
                            let (first_access,_) = access_information.1.as_ref().unwrap().get(0).unwrap();
                            let output = program_archive.get_template_data(&template_name).get_output_info(&first_access);
                            if output.is_some() {
                                return add_report( ReportCode::OutputTagCannotBeModifiedOutside,meta, &mut analysis_information.reports);
                            }
                            let input = program_archive.get_template_data(&template_name).get_input_info(&first_access);
                            if input.is_some() {
                                return add_report( ReportCode::InputTagCannotBeModifiedOutside,meta, &mut analysis_information.reports);
                            }
                        }
                    }
                }
                (SymbolInformation::Signal(_), AssignOp::AssignVar) => {
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
                SymbolInformation::Component(possible_template) => {
                    if rhe_type.is_template() {
                        if possible_template.is_none() {
                            let (current_template, _) = analysis_information
                                .environment
                                .get_mut_component_or_break(var, file!(), line!());
                            *current_template = rhe_type.template;
                        } else {
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
                    } else {
                        add_report(
                            ReportCode::WrongTypesInAssignOperationTemplate,
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                }
                SymbolInformation::Bus(possible_bus, dim) => {
                    if rhe_type.is_bus() {
                        if dim != rhe_type.dim() {
                            add_report(
                                ReportCode::WrongTypesInAssignOperationDims(dim, rhe_type.dim()),
                                meta,
                                &mut analysis_information.reports,
                            )
                        }
                        else if possible_bus.is_none() {
                            let (current_bus, _,_) = analysis_information
                                .environment
                                .get_mut_bus_or_break(var, file!(), line!());
                            *current_bus = rhe_type.bus;
                        } else {
                            let bus = possible_bus.unwrap();
                            let r_bus = rhe_type.bus.unwrap();
                            if bus != r_bus {
                                if dim > 0 {
                                    add_report(
                                        ReportCode::WrongTypesInAssignOperationArrayBuses,
                                        meta,
                                        &mut analysis_information.reports,
                                    )
                                }
                                else {
                                    add_report(
                                        ReportCode::WrongTypesInAssignOperationBus,
                                        meta,
                                        &mut analysis_information.reports,
                                    )
                                }
                            }
                        }
                    } else {
                        add_report(
                            ReportCode::WrongTypesInAssignOperationBus,
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                }
                SymbolInformation::Signal(dim)
                | SymbolInformation::Var(dim) => {
                    if rhe_type.is_template()  {
                        add_report(
                            ReportCode::WrongTypesInAssignOperationExpression,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if  rhe_type.is_bus() {
                        add_report(
                            ReportCode::WrongTypesInAssignOperationBus,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if dim != rhe_type.dim() {
                        add_report(
                            ReportCode::WrongTypesInAssignOperationDims(dim, rhe_type.dim()),
                            meta,
                            &mut analysis_information.reports,
                        )
                    }
                }
                SymbolInformation::Tag => {
                    if rhe_type.is_template()  {
                        add_report(
                            ReportCode::WrongTypesInAssignOperationExpression,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if  rhe_type.is_bus() {
                        add_report(
                            ReportCode::WrongTypesInAssignOperationBus,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if 0 != rhe_type.dim() {
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
            if lhe_type.is_bus() || rhe_type.is_bus() {
                match (lhe_type.bus, rhe_type.bus) {
                    (Some(b1),Some(b2)) => {
                        if b1 != b2 {
                            add_report(ReportCode::MustBeSameBus, lhe.get_meta(), 
                                    &mut analysis_information.reports);
                        }
                    }
                    (Some(_),_)=>{
                        add_report(ReportCode::MustBeBus, rhe.get_meta(), 
                        &mut analysis_information.reports);
                    },
                    (_,Some(_)) => {
                        add_report(ReportCode::MustBeBus, lhe.get_meta(), 
                        &mut analysis_information.reports);
                    },
                    (_,_) => {unreachable!("At least one of them is a bus.")}
                }
            }
            else if rhe_type.dim() != lhe_type.dim() {
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
                    if arg_type.is_template() {
                        add_report(
                            ReportCode::MustBeSingleArithmeticT,
                            meta,
                            &mut analysis_information.reports,
                        )
                    } else if arg_type.is_bus() {
                        add_report(
                            ReportCode::MustBeSingleArithmeticB,
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
            } else if arg_type.is_bus() {
                add_report(
                    ReportCode::MustBeSingleArithmeticB,
                    meta,
                    &mut analysis_information.reports,
                )
            }  else if arg_type.dim() > 0 {
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
            if cond_type.is_template() {
                add_report(
                    ReportCode::MustBeSingleArithmeticT,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            } else if cond_type.is_bus() {
                add_report(
                    ReportCode::MustBeSingleArithmeticB,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            }  else if cond_type.dim() > 0 {
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
            if cond_type.is_template() {
                add_report(
                    ReportCode::MustBeSingleArithmeticT,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            } else if cond_type.is_bus() {
                add_report(
                    ReportCode::MustBeSingleArithmeticB,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            } else if cond_type.dim() > 0 {
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
                } else if value_type.is_bus() {
                    add_report(
                        ReportCode::InvalidArrayTypeB,
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
            } else if dim_type.is_bus() {
                add_report(
                    ReportCode::InvalidArrayTypeB,
                    expression.get_meta(),
                    &mut analysis_information.reports,
                );
            }else if dim_type.dim() != 0 {
                add_report(
                    ReportCode::InvalidArrayType,
                    expression.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            if let Some(iden) = &value_type.bus {
                Result::Ok(FoldedType::bus(iden.clone().as_str(), value_type.dim() + 1))
            } else {
                Result::Ok(FoldedType::arithmetic_type(value_type.dim() + 1))
            }
        }
        InfixOp { lhe, rhe, .. } => {
            let lhe_response = type_expression(lhe, program_archive, analysis_information);
            let rhe_response = type_expression(rhe, program_archive, analysis_information);
            let lhe_type = lhe_response?;
            let rhe_type = rhe_response?;
            let mut successful = Result::Ok(());
            if lhe_type.is_template() || lhe_type.is_bus() || lhe_type.dim() > 0 {
                successful = add_report_and_end(
                    ReportCode::InfixOperatorWithWrongTypes,
                    lhe.get_meta(),
                    &mut analysis_information.reports,
                );
            }
            if rhe_type.is_template() || rhe_type.is_bus() || rhe_type.dim() > 0 {
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
            if rhe_type.is_template() || rhe_type.is_bus() || rhe_type.dim() > 0 {
                add_report_and_end(
                    ReportCode::PrefixOperatorWithWrongTypes,
                    rhe.get_meta(),
                    &mut analysis_information.reports,
                )
            } else {
                Result::Ok(FoldedType::arithmetic_type(0))
            }
        }
        ParallelOp {rhe, .. } => {
            let rhe_type = type_expression(rhe, program_archive, analysis_information)?;
            if rhe_type.is_template() {
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
            let if_false_response = type_expression(if_false, program_archive, analysis_information);
            let if_true_type = if_true_response?;

            let cond_type = if let Result::Ok(f) = cond_response {
                f
            } else {
                return Result::Ok(if_true_type);
            };
            if cond_type.is_template() {
                add_report(
                    ReportCode::MustBeSingleArithmeticT,
                    cond.get_meta(),
                    &mut analysis_information.reports,
                )
            } else if cond_type.is_bus() {
                add_report(
                    ReportCode::MustBeSingleArithmeticB,
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
                treat_access( access, program_archive, analysis_information)?;
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
                SymbolInformation::Component(possible_template) if possible_template.is_none() => {
                    add_report_and_end(ReportCode::UninitializedSymbolInExpression, meta, reports)
                }
                SymbolInformation::Bus(possible_bus, dim) if possible_bus.is_some() => {
                    Result::Ok(FoldedType::bus(&possible_bus.unwrap(), dim))
                }
                SymbolInformation::Bus(possible_bus, _) if possible_bus.is_none() => {
                    add_report_and_end(ReportCode::UninitializedSymbolInExpression, meta, reports)
                }
                SymbolInformation::Var(dim) | SymbolInformation::Signal(dim) => {
                    Result::Ok(FoldedType::arithmetic_type(dim))
                }
                SymbolInformation::Tag => {
                    Result::Ok(FoldedType::arithmetic_type(0))
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
        },
        BusCall { meta, id, args } => {
            analysis_information.reached.insert(id.clone());
            let typing_response =
                type_array_of_expressions(args, program_archive, analysis_information);
            if program_archive.contains_bus(id) && typing_response.is_err() {
                return Result::Ok(FoldedType::bus(id,program_archive.get_bus_data(id).get_fields().len()));
            }
            let arg_types = typing_response?;
            let mut concrete_types = Vec::new();
            let mut success = Result::Ok(());
            for (arg_expr, arg_type) in args.iter().zip(arg_types.iter()) {
                if arg_type.is_template() {
                    success = add_report_and_end(
                        ReportCode::InvalidArgumentInBusInstantiationT,
                        arg_expr.get_meta(),
                        &mut analysis_information.reports,
                    );
                } else if arg_type.is_bus() {
                    success = add_report_and_end(
                        ReportCode::InvalidArgumentInBusInstantiationB,
                        arg_expr.get_meta(),
                        &mut analysis_information.reports,
                    );
                }
                concrete_types.push(arg_type.dim());
            }
            if program_archive.contains_bus(id) && success.is_err() {
                return Result::Ok(FoldedType::bus(id,program_archive.get_bus_data(id).get_fields().len()));
            }
            success?;
            let previous_file_id = analysis_information.file_id;
            analysis_information.file_id = program_archive.get_bus_data(id).get_file_id();

            let new_environment = prepare_environment_for_call(
                meta,
                id,
                &concrete_types,
                program_archive,
                &mut analysis_information.reports,
            );
            if new_environment.is_err() {
                return Result::Ok(FoldedType::bus(id,program_archive.get_bus_data(id).get_fields().len()));
            }
            let new_environment = new_environment?;
            let previous_environment =
                std::mem::replace(&mut analysis_information.environment, new_environment);
            let returned_type = type_bus(id, &concrete_types, analysis_information, program_archive);
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
// 1: Vector of (Wire accessed and dimensions accessed in that wire) (optional)
//                 Size of this vector is equals to the number of buses (+1 if it finishes in a signal))
//                                                                        (+1 if it finishes in a tag)
type AccessInfo = (ArithmeticType, Option<Vec<(String, ArithmeticType)>>);
fn treat_access(
    accesses: &[Access],
    program_archive: &ProgramArchive,
    analysis_information: &mut AnalysisInformation,
) -> Result<AccessInfo, ()> {
    use Access::*;
    let mut access_info: AccessInfo = (0, Option::None);
    let mut signal_info : Vec<(String, ArithmeticType)> = Vec::new();
    for access in accesses {
        match access {
            ArrayAccess(index) => {
                let index_response = type_expression(&index, program_archive, analysis_information);
                if signal_info.len() > 0 {
                    let mut info = signal_info.get(signal_info.len()-1).unwrap().clone();
                    info.1 = info.1 + 1;
                    signal_info.remove(signal_info.len()-1);
                    signal_info.push(info);
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
                    } else if index_type.is_bus() {
                        add_report(
                                ReportCode::InvalidArraySizeB,
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
            ComponentAccess(name) => {

                if signal_info.len() > 0 {
                    signal_info.push((name.clone(),0));
                } else {
                    signal_info = vec![(name.clone(), 0)];
                }
            }
        }
    }
    if signal_info.len() > 0{
        access_info.1 = Some(signal_info);
    } else { access_info.1 = None; }
    Result::Ok(access_info)
}

enum SymbolInformation {
    Component(Option<String>),
    Var(ArithmeticType),
    Signal(ArithmeticType),
    Bus(Option<String>, ArithmeticType),
    Tag,
}

fn check_if_it_is_a_tag(
    symbol: &str,
    meta: &Meta,
    access_information: AccessInfo,
    environment: &TypingEnvironment,
    reports: &mut ReportCollection,
    program_archive: &ProgramArchive,
) -> Result<bool, ()> {
    let buses_and_signals = if access_information.1.is_none() { // no access
        return Result::Ok(false);
    } else {
        access_information.1.unwrap()
    };
    let mut num_dims_accessed = 0;
    let mut pos = 0;
    let mut it_is_input_subcomponent = false;
    let (mut kind, mut tags) = if environment.has_component(symbol){ 
        // we are inside component
        let (name, dim) = environment.get_component_or_break(symbol, file!(), line!()).clone();
        let current_dim = dim - access_information.0;
        if current_dim != 0{ // only allowed complete accesses to component
            return add_report_and_end(ReportCode::InvalidPartialArray, meta, reports);
        } else if buses_and_signals.len() <= 1 {
            return Result::Ok(false);
        }
        //current_dim == 0 => component completely defined
        //buses_and_signals.len() > 1 => we are accessing a signal, or a bus and later maybe a tag
        let template_name = name.unwrap();
        let (accessed_element,accessed_dim) = buses_and_signals.get(0).unwrap();
        pos += 1;
        num_dims_accessed += accessed_dim; 
        let input = program_archive.get_template_data(&template_name).get_input_info(&accessed_element);
        let output = program_archive.get_template_data(&template_name).get_output_info(&accessed_element);
        let (kind, atags) = if let Some(wire_data) = input {
            it_is_input_subcomponent = true;
            (wire_data.get_type(), wire_data.get_tags())
        } else if let Some(wire_data) = output {
            (wire_data.get_type(), wire_data.get_tags())
        } else {
            return add_report_and_end(ReportCode::InvalidSignalAccess, meta, reports);
        };
        (kind, atags.clone())
    } 
    else { // we are outside component
        num_dims_accessed = access_information.0;
        let (kind, possible_tags) = if environment.has_variable(symbol) { return Result::Ok(false); }
        else if environment.has_bus(symbol){
            let (current_bus,_,possible_tags) = environment.get_bus_or_break(symbol, file!(), line!()).clone();
            let kind = WireType::Bus(current_bus.unwrap());
            (kind, possible_tags)
        } else {
            let kind = WireType::Signal;
            let (_, possible_tags) = environment.get_signal_or_break(symbol, file!(), line!()).clone();
            (kind,possible_tags)
        };
        let mut tags = HashSet::new();
        for i in possible_tags.clone() {
            tags.insert(i);
        }
        (kind, tags) };
    while pos < buses_and_signals.len() {
        let (accessed_element, accessed_dim) = buses_and_signals.get(pos).unwrap().clone();
        num_dims_accessed += accessed_dim;
        if kind == WireType::Signal {
                if tags.contains(&accessed_element) {

                    //Tags cannot be partially accessed. Then, the previous bus or signal cannot be array accessed.
                    if pos == buses_and_signals.len()-1 && num_dims_accessed == 0{
                        return if it_is_input_subcomponent { add_report_and_end(ReportCode::InputTagCannotBeAccessedOutside, meta, reports)} 
                                else {Result::Ok(true)};
                    } else if num_dims_accessed > 0 {
                        return add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports);
                    } else{
                            return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                    }
                }
                else{
                    return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                }
        } else if let WireType::Bus(b_name) = kind  {
                let field = program_archive.get_bus_data(&b_name).get_field_info(&accessed_element);
                match field {
                    Some(wire) => {
                        if pos == buses_and_signals.len()-1 {
                            return Result::Ok(false);
                        } else {
                            kind = wire.get_type();
                            tags = wire.get_tags().clone();
                        }
                    },
                    Option::None => {
                        if tags.contains(&accessed_element) {
                            if pos == buses_and_signals.len()-1 && num_dims_accessed == 0{
                                return if it_is_input_subcomponent { add_report_and_end(ReportCode::InputTagCannotBeAccessedOutside, meta, reports)} 
                                       else {Result::Ok(true)};
                            } else if num_dims_accessed > 0 {
                                return add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports);
                            } else{
                                    return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                            }
                        }
                        else {
                            return Ok(false);
                        }
                    },
            }
        } else{
            unreachable!()
        }
        pos += 1;
    }
    return Ok(false);
}

fn apply_access_to_symbol(
    symbol: &str,
    meta: &Meta,
    access_information: AccessInfo,
    environment: &TypingEnvironment,
    reports: &mut ReportCollection,
    program_archive: &ProgramArchive,
) -> Result<SymbolInformation, ()> {
    let it_is_tag = check_if_it_is_a_tag(symbol, meta, access_information.clone(), environment, reports, program_archive)?;
    if it_is_tag {
        return Result::Ok(SymbolInformation::Tag);
    }
    let (current_template_or_bus, mut current_dim, possible_tags) = if environment.has_component(symbol) {
        let (temp, dim) = environment.get_component_or_break(symbol, file!(), line!()).clone();
        (temp.clone(),dim, Vec::new())
    } else if environment.has_signal(symbol) {
        let(dim, tags) = environment.get_signal_or_break(symbol, file!(), line!());
        (Some(symbol.to_string()),  *dim, tags.clone())
    } else if environment.has_bus(symbol){
        environment.get_bus_or_break(symbol, file!(), line!()).clone()
    } else {
        let dim = environment.get_variable_or_break(symbol, file!(), line!());
        (Option::None, *dim, Vec::new())
    };
    if access_information.0 > current_dim {
        return add_report_and_end(ReportCode::InvalidArrayAccess(current_dim, access_information.0), meta, reports);
    } else {
        current_dim -= access_information.0
    }
    // Case wires or tags 
     if let Option::Some(buses_and_signals) = access_information.1{
        assert!(buses_and_signals.len() > 0);
        let mut pos = 0;
        if current_dim > 0 && (pos < buses_and_signals.len()- 1 || !possible_tags.contains(&buses_and_signals.get(0).unwrap().0)){
            return add_report_and_end(ReportCode::InvalidArrayAccess(current_dim+access_information.0,access_information.0), meta, reports);
        }
        if current_template_or_bus.is_none() && environment.has_bus(symbol){
            return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
        } else if current_template_or_bus.is_none() && environment.has_component(symbol){
            return add_report_and_end(ReportCode::UninitializedComponent, meta, reports);
        }

        let (mut kind, mut tags) = if environment.has_component(symbol){ 
            // we are inside component
            
            if current_dim != 0{ // only allowed complete accesses to component
                return add_report_and_end(ReportCode::InvalidPartialArray, meta, reports);
            }
            //current_dim == 0 => component completely defined
            let template_name = current_template_or_bus.unwrap();
            let (accessed_element,accessed_dim) = buses_and_signals.get(pos).unwrap();
            let input = program_archive.get_template_data(&template_name).get_input_info(&accessed_element);
            let output = program_archive.get_template_data(&template_name).get_output_info(&accessed_element);
            let (dim, kind, atags) = match (input, output) {
                (Option::Some(wire_data), _) | (_, Option::Some(wire_data)) =>
                    (wire_data.get_dimension(), wire_data.get_type(), wire_data.get_tags()),
                _ => {
                    return add_report_and_end(ReportCode::InvalidSignalAccess, meta, reports);
                }
            };
            
            if *accessed_dim > dim {
                return add_report_and_end(ReportCode::InvalidArrayAccess(dim, *accessed_dim), meta, reports);
            }
            current_dim = dim - accessed_dim;
            if pos == buses_and_signals.len()-1 {
                match kind {
                    WireType::Signal => {return Result::Ok(SymbolInformation::Signal(current_dim));},
                    WireType::Bus(b_name) => {return Result::Ok(SymbolInformation::Bus(Some(b_name.clone()),current_dim))},
                }
            }
            pos += 1;
            (kind, atags.clone())
        } else if environment.has_bus(symbol){
            let kind = WireType::Bus(current_template_or_bus.unwrap());
            let mut tags = HashSet::new();
            for i in possible_tags.clone() {
                tags.insert(i);
            }
            (kind, tags)
        } else {
            let kind = WireType::Signal;
            let mut tags = HashSet::new();
            for i in possible_tags.clone() {
                tags.insert(i);
            }
            (kind,tags)
        };
        while pos < buses_and_signals.len() {
            let (accessed_element, accessed_dim) = buses_and_signals.get(pos).unwrap().clone();
            if current_dim > 0 && (pos < buses_and_signals.len()- 1 || !tags.contains(&accessed_element)){
                return add_report_and_end(ReportCode::InvalidArrayAccess(current_dim,accessed_dim), meta, reports);
            }
            if kind == WireType::Signal {
                    if tags.contains(&accessed_element) {
                        let prev_dim_access = if buses_and_signals.len()>1 {buses_and_signals.get(buses_and_signals.len()-2).unwrap().1}
                                                  else {access_information.0}; 
                        //Tags cannot be partially accessed. Then, the previous bus or signal cannot be array accessed.
                        if pos == buses_and_signals.len()-1 && 0 == prev_dim_access && accessed_dim == 0{
                            return Result::Ok(SymbolInformation::Tag);
                        } else if prev_dim_access > 0 {
                            return add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports);
                        } else{
                                return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                        }
                    }
                    else{
                        return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                    }
            } else if let WireType::Bus(b_name) = kind  {
                    let field = program_archive.get_bus_data(&b_name).get_field_info(&accessed_element);
                    match field {
                        Some(wire) => {
                            if accessed_dim > wire.get_dimension() {
                                return add_report_and_end(ReportCode::InvalidArrayAccess(wire.get_dimension(), accessed_dim), meta, reports);
                            }
                            current_dim = wire.get_dimension() - accessed_dim;
                            if pos == buses_and_signals.len()-1 {
                                match wire.get_type() {
                                    WireType::Signal => {return Result::Ok(SymbolInformation::Signal(current_dim));},
                                    WireType::Bus(b_name2) => {return Result::Ok(SymbolInformation::Bus(Some(b_name2.clone()), current_dim))},
                                }
                            } else {
                                kind = wire.get_type();
                                tags = wire.get_tags().clone();
                            }
                        },
                        Option::None => {
                            if tags.contains(&accessed_element) {
                                let prev_dim_access = if buses_and_signals.len()>1 {buses_and_signals.get(buses_and_signals.len()-2).unwrap().1}
                                                            else {access_information.0}; 
                                if pos == buses_and_signals.len()-1 && prev_dim_access == 0 && accessed_dim == 0{
                                    return Result::Ok(SymbolInformation::Tag);
                                } else if prev_dim_access > 0 {
                                    return add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports);
                                } else{
                                        return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                                }
                            }
                            else{
                                return add_report_and_end(ReportCode::InvalidTagAccess, meta, reports);
                            }
                        },
                }
            } else{
                unreachable!()
            }
            pos += 1;
        }
        unreachable!()


        //add_report_and_end(ReportCode::InvalidTagAccessAfterArray, meta, reports);
    } else if environment.has_variable(symbol) {
        Result::Ok(SymbolInformation::Var(current_dim))
    } else if environment.has_signal(symbol) {
        Result::Ok(SymbolInformation::Signal(current_dim))
    } else if environment.has_bus(symbol){ 
        Result::Ok(SymbolInformation::Bus(current_template_or_bus, current_dim))
    }else if environment.has_component(symbol) && current_dim == 0 {
        Result::Ok(SymbolInformation::Component(current_template_or_bus))
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
    } else if program_archive.contains_template(call_id) {
        program_archive.get_template_data(call_id).get_name_of_params()
    } else {
        program_archive.get_bus_data(call_id).get_name_of_params()
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


fn type_bus(id: &str, args_dims: &[ArithmeticType], analysis_information: &mut AnalysisInformation, program_archive: &ProgramArchive) -> Result<FoldedType,()> {
    debug_assert!(program_archive.contains_bus(id));
    if analysis_information.registered_calls.get_instance(id, args_dims).is_none() {
        analysis_information.registered_calls.add_instance(id, args_dims.to_vec(), 0);
        let stmts = program_archive.get_bus_data(id).get_body_as_vec();
        treat_sequence_of_statements(stmts, program_archive, analysis_information);
    }
    Result::Ok(FoldedType::bus(id,0))
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
        InvalidArraySizeB =>{
            "Array indexes and lengths must be single arithmetic expressions.\n Found bus instead of expression.".to_string()
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
        InvalidArrayTypeB => "Buses can not be declared inside inline arrays".to_string(),
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
        WrongTypesInAssignOperationTemplate => "Assignee and assigned types do not match.\n Expected template but found expression.".to_string(),
        WrongTypesInAssignOperationArrayBuses => "Assignee and assigned types do not match.\n All buses of an array must be the same type.".to_string(),
        WrongTypesInAssignOperationBus => "Assignee and assigned types do not match.\n Expected bus but found a different expression.".to_string(),
        WrongTypesInAssignOperationExpression => "Assignee and assigned types do not match.\n Expected expression found template.".to_string(),
        WrongTypesInAssignOperationDims(expected, found) => {
            format!("Assignee and assigned types do not match. \n Expected dimensions: {}, found {}",
            expected, found)
        }
        InvalidArgumentInCall => "Components can not be passed as arguments".to_string(),
        InvalidArgumentInBusInstantiationT => "Components can not be passed as arguments".to_string(),
        InvalidArgumentInBusInstantiationB => "Buses can not be passed as arguments".to_string(),
        UnableToTypeFunction => "Unable to infer the type of this function".to_string(),
        MustBeSingleArithmetic(dim) => {
            format!("Must be a single arithmetic expression.\n Found expression of {} dimensions", dim)
        }
        MustBeSingleArithmeticT => {
              format!("Must be a single arithmetic expression.\n Found component")
        }
        MustBeSingleArithmeticB => format!("Must be a single arithmetic expression.\n Found bus"),
        MustBeArithmetic => "Must be a single arithmetic expression or an array of arithmetic expressions. \n Found component".to_string(),
        OutputTagCannotBeModifiedOutside => "Output tag from a subcomponent cannot be modified".to_string(),
        InputTagCannotBeModifiedOutside => "Input tag from a subcomponent cannot be modified".to_string(),
        InputTagCannotBeAccessedOutside => "Input tag from a subcomponent cannot be accessed".to_string(),
        MustBeSameDimension(dim_1, dim_2) =>{
            format!("Must be two arrays of the same dimensions.\n Found {} and {} dimensions", dim_1, dim_2)
        }
        MainComponentWithTags => "Main component cannot have inputs with tags".to_string(),
        ExpectedDimDiffGotDim(expected, got) => {
            format!("All branches of a function should return an element of the same dimensions.\n Found {} and {} dimensions", expected, got)
        }
        WrongNumberOfArguments(expected, got) => {
            format!("Expecting {} arguments, {} where obtained", expected, got)
        }
        UninitializedComponent => "Trying to access to a signal of a component that has not been initialized".to_string(),
        NonCompatibleBranchTypes => "Inline switch operator branches types are non compatible".to_string(),
        MustBeSameBus => "Both kind of buses must be equals".to_string(),
        MustBeBus => "Expected to be a bus".to_string(),
        InvalidSignalAccessInBus => format!("Field not defined in bus"),
        IllegalMainExpression => "Invalid main component: the main component should be a template, not a function call or expression".to_string(),
        e => panic!("Unimplemented error code: {}", e),
    };
    report.add_primary(location, file_id, message);
    reports.push(report);
}