use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::program_archive::ProgramArchive;
use program_structure::wire_data::WireType;
use program_structure::ast::*;
use program_structure::bus_data::BusData;
use program_structure::environment::CircomEnvironment;
use program_structure::function_data::FunctionData;
use program_structure::template_data::TemplateData;

type Environment = CircomEnvironment<TypeKnowledge, (), (), TypeKnowledge>;

pub fn reduce_function(function_data: &mut FunctionData,  program_archive : &ProgramArchive)  -> ReportCollection {
    let mut environment = Environment::new();
    for param in function_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = function_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment, program_archive)
}
pub fn reduce_template(template_data: &mut TemplateData, program_archive : &ProgramArchive) -> ReportCollection  {
    let mut environment = Environment::new();
    for param in template_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = template_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment, program_archive)
}

pub fn reduce_bus(bus_data: &mut BusData,  program_archive : &ProgramArchive)  -> ReportCollection  {
    let mut environment = Environment::new();
    for param in bus_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = bus_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment, program_archive)
}

fn reduce_types_in_statement(stmt: &mut Statement, environment: &mut Environment,  program_archive : &ProgramArchive)  -> ReportCollection {
    use Statement::*;
    match stmt {
        Substitution { var, access, rhe, meta, .. } => {
            reduce_types_in_substitution(var, access, environment, rhe, meta, program_archive)
        }
        Declaration { name, xtype, dimensions, .. } => {
            reduce_types_in_declaration(xtype, name, dimensions, environment,program_archive)
        }
        While { cond, stmt, .. } => reduce_types_in_while(cond, stmt, environment,program_archive),
        Block { stmts, .. } => reduce_types_in_vec_of_statements(stmts, environment,program_archive),
        InitializationBlock { initializations, .. } => {
            reduce_types_in_vec_of_statements(initializations, environment,program_archive)
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            reduce_types_in_conditional(cond, if_case, else_case, environment,program_archive)
        }
        LogCall { args, .. } => {
                reduce_types_in_log_call(args, environment,program_archive)
            
        },
        Assert { arg, .. } => reduce_types_in_expression(arg, environment,program_archive),
        Return { value, .. } => reduce_types_in_expression(value, environment,program_archive),
        ConstraintEquality { lhe, rhe, .. } => {
            reduce_types_in_constraint_equality(lhe, rhe, environment,program_archive)
        }
        MultSubstitution { .. } => unreachable!(),
        UnderscoreSubstitution { rhe, .. } => {
            reduce_types_in_expression(rhe, environment,program_archive)
        },
    }
}

fn reduce_types_in_log_call(args: &mut Vec<LogArgument>, environment: &Environment, program_archive : &ProgramArchive)
    -> ReportCollection {
    let mut reports = Vec::new();
    for arg in args {
        if let LogArgument::LogExp(exp) = arg {
            reports.append(&mut reduce_types_in_expression(exp, environment, program_archive));
        }
    }
    reports
}

fn reduce_types_in_expression(expression: &mut Expression, environment: &Environment, program_archive : &ProgramArchive) 
 -> ReportCollection {
    use Expression::*;
    match expression {
        Variable { name, access, meta, .. } => {
            reduce_types_in_variable(name, environment, access, meta,program_archive)
        }
        InfixOp { lhe, rhe, .. } => reduce_types_in_infix(lhe, rhe, environment,program_archive),
        PrefixOp { rhe, .. } => reduce_types_in_expression(rhe, environment,program_archive),
        ParallelOp { meta, rhe, .. } => {
            let report = reduce_types_in_expression(rhe, environment,program_archive);
            meta.get_mut_type_knowledge().set_reduces_to(rhe.get_meta().get_type_knowledge().get_reduces_to());
            report
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            reduce_types_in_inline_switch(cond, if_true, if_false, environment,program_archive)
        }
        Call { args, id, meta, .. } => {
            meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Component(Some(id.clone())));
            reduce_types_in_vec_of_expressions(args, environment,program_archive)
        },
        ArrayInLine { values, .. } => reduce_types_in_vec_of_expressions(values, environment,program_archive),
        UniformArray { value, dimension, .. } => {
            let mut reports = reduce_types_in_expression(value, environment,program_archive);
            reports.append(&mut reduce_types_in_expression(dimension, environment,program_archive));
            reports
        }
        Number(..) => { Vec::new() }
        BusCall { args, meta, id, .. } => {
            meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Bus(Some(id.clone())));
            reduce_types_in_vec_of_expressions(args, environment,program_archive)
        },
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}

fn reduce_types_in_constraint_equality(
    lhe: &mut Expression,
    rhe: &mut Expression,
    environment: &mut Environment,
    program_archive : &ProgramArchive
) -> ReportCollection{
    let mut reports = reduce_types_in_expression(lhe, environment,program_archive);
    reports.append(&mut reduce_types_in_expression(rhe, environment,program_archive));
    reports 
}

fn reduce_types_in_declaration(
    xtype: &VariableType,
    name: &str,
    dimensions: &mut [Expression],
    environment: &mut Environment,
    program_archive : &ProgramArchive
)  -> ReportCollection  {
    use VariableType::*;
    if *xtype == Var {
        environment.add_variable(name, ());
    } else if *xtype == Component || *xtype == AnonymousComponent {
        let mut typ = TypeKnowledge::default(); 
        typ.set_reduces_to(TypeReduction::Component(None));
        environment.add_component(name, typ);
    } else if let Bus(bname,_,_) = xtype.clone(){
        let mut typ = TypeKnowledge::default(); 
        typ.set_reduces_to(TypeReduction::Bus(Some(bname)));
        environment.add_intermediate_bus(name, typ)
    } else {
        environment.add_intermediate(name, ());
    }
    reduce_types_in_vec_of_expressions(dimensions, environment,program_archive)
}

fn reduce_types_in_substitution (
    name: &str,
    access: &mut [Access],
    environment: &mut Environment,
    expr: &mut Expression,
    meta: &mut Meta,
    program_archive : &ProgramArchive
)  -> ReportCollection {
    let mut reports = reduce_types_in_variable(name, environment, access, meta,program_archive);
    reports.append(&mut reduce_types_in_expression(expr, environment,program_archive));
    let mut is_simple_component = true;
    for a in access{
        if let Access::ComponentAccess(_) = a  {
            is_simple_component = false;
        }
    }
    if is_simple_component {
        let xtype = environment.get_mut_component(name); 
        if xtype.is_some() && xtype.as_ref().unwrap().is_initialized() {
            let type_knowledge = expr.get_meta().get_type_knowledge();
            if type_knowledge.is_initialized() {
                let reduced_type = expr.get_meta().get_type_knowledge().get_reduces_to();
                xtype.unwrap().set_reduces_to(reduced_type);
            }
        }
    }
    reports 
}

fn reduce_types_in_while(
    cond: &mut Expression,
    stmt: &mut Statement,
    environment: &mut Environment,
    program_archive : &ProgramArchive
)-> ReportCollection{
    let mut reports = Vec::new();
    reports.append(&mut reduce_types_in_expression(cond, environment,program_archive));
    reports.append(&mut reduce_types_in_statement(stmt, environment, program_archive));
    reports 
}

fn reduce_types_in_conditional(
    cond: &mut Expression,
    if_branch: &mut Statement,
    else_branch: &mut Option<Box<Statement>>,
    environment: &mut Environment,
    program_archive : &ProgramArchive
) -> ReportCollection {
    let mut reports = Vec::new();
    reports.append(&mut reduce_types_in_expression(cond, environment,program_archive));
    reports.append(&mut reduce_types_in_statement(if_branch, environment,program_archive));
    if let Option::Some(else_stmt) = else_branch {
        reports.append(&mut reduce_types_in_statement(else_stmt, environment,program_archive));
    }
    reports
}

fn reduce_types_in_vec_of_statements(vec: &mut [Statement], environment: &mut Environment, program_archive : &ProgramArchive) 
    -> ReportCollection {
    let mut reports = Vec::new();
    for stmt in vec {
        reports.append(&mut reduce_types_in_statement(stmt, environment,program_archive));
    }
    reports
}

fn reduce_types_in_variable(
    oname: &str,
    environment: &Environment,
    access: &mut [Access],
    meta: &mut Meta,
    program_archive : &ProgramArchive
)  -> ReportCollection  {
    use Access::*;
    use TypeReduction::*;
    let mut reports = Vec::new();
    let mut reduction = if environment.has_signal(oname) {
        Signal
    } else if environment.has_component(oname) {
        environment.get_component(oname).unwrap().get_reduces_to()
    } else  if environment.has_bus(oname){
        environment.get_bus(oname).unwrap().get_reduces_to()
    } else {
        Variable
    };

    for acc in access {
        match acc {
            ComponentAccess(name) => {
                match reduction{
                    Variable => {}, //Check type will return the corresponding error. 
                    Component(ref comp) => {
                        if let Some(comp) = comp {
                            let template = program_archive.get_template_data(comp.as_str());
                            let wire = template.get_inputs().get(name).or(template.get_outputs().get(name));
                            if wire.is_some() {
                                match wire.unwrap().get_type(){
                                    WireType::Signal => reduction = Signal,
                                    WireType::Bus(new_name) => reduction = Bus(Some(new_name)),
                             }//If it is not a signal or a bus, it is expected to be tag. 
                            } else {//Then, type_check will finally check it.                          
                                name_not_found_in_component_error(name.clone(), oname.to_string(), meta,&mut  reports); 
                                return reports;  
                            }
                        }
                    },
                    Bus(ref b) => {
                        if let Some(b) = b {
                            let busdata = program_archive.get_bus_data(b.as_str());
                            if let Some(wire) = busdata.get_fields().get(name){ 
            
                                match wire.get_type(){
                                    WireType::Signal => reduction = Signal,
                                    WireType::Bus(new_name) => reduction = Bus(Some(new_name)),
                                }
                            } else {
                                reduction = Tag;
                            }
                        }
                    },
                    Signal => reduction = Tag,
                    Tag => {},
                }
            },
            ArrayAccess(exp) => {
                reports.append(&mut reduce_types_in_expression(exp, environment,program_archive));
            },
        }
    }
    meta.get_mut_type_knowledge().set_reduces_to(reduction);
    reports
}

fn reduce_types_in_infix(lhe: &mut Expression, rhe: &mut Expression, environment: &Environment,  program_archive : &ProgramArchive) -> ReportCollection{
    let mut reports = Vec::new();
    reports.append(&mut reduce_types_in_expression(lhe, environment, program_archive));
    reports.append(&mut reduce_types_in_expression(rhe, environment, program_archive));
    reports
}

fn reduce_types_in_inline_switch(
    cond: &mut Expression,
    if_true: &mut Expression,
    if_false: &mut Expression,
    environment: &Environment,
    program_archive : &ProgramArchive
)  -> ReportCollection {
    let mut reports = Vec::new();
    reports.append(&mut reduce_types_in_expression(cond, environment,program_archive));
    reports.append(&mut reduce_types_in_expression(if_true, environment,program_archive));
    reports.append(&mut reduce_types_in_expression(if_false, environment,program_archive));
    reports
}

fn reduce_types_in_vec_of_expressions(vec: &mut [Expression], environment: &Environment, program_archive : &ProgramArchive)  -> ReportCollection {
    let mut reports = Vec::new();
    for expr in vec {
        reports.append(& mut reduce_types_in_expression(expr, environment,program_archive));
    }
    reports
}

// Errors
// fn name_not_found_in_bus_error(signal: String, what: String, meta: &Meta, reports: &mut ReportCollection) {
//     let message = "Bus or signal not defined in bus".to_string();
//     let error_code = ReportCode::InvalidSignalAccessInBus;
//     let mut report = Report::error(message, error_code);
//     let message = signal + &" is not defined in ".to_string() + what.as_str();
//     report.add_primary(meta.file_location(), meta.get_file_id(), message);
//     reports.push(report);
// }

fn name_not_found_in_component_error(signal: String, what: String, meta: &Meta, reports: &mut ReportCollection) {
    let message = "Bus or signal not defined in component".to_string();
    let error_code = ReportCode::InvalidSignalAccess;
    let mut report = Report::error(message, error_code);
    let message = signal + &" is not defined in ".to_string() + what.as_str();
    report.add_primary(meta.file_location(), meta.get_file_id(), message);
    reports.push(report);
}