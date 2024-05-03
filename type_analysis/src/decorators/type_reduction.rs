use program_structure::program_archive::{self, ProgramArchive};
use program_structure::wire_data::WireType;
use program_structure::{ast::*, bus_data};
use program_structure::bus_data::BusData;
use program_structure::environment::CircomEnvironment;
use program_structure::function_data::FunctionData;
use program_structure::template_data::TemplateData;

type Environment = CircomEnvironment<TypeKnowledge, (), (), TypeKnowledge>;

pub fn reduce_function(function_data: &mut FunctionData,  program_archive : &ProgramArchive) {
    let mut environment = Environment::new();
    for param in function_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = function_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment, program_archive);
}
pub fn reduce_template(template_data: &mut TemplateData, program_archive : &ProgramArchive) {
    let mut environment = Environment::new();
    for param in template_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = template_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment, program_archive);
}

pub fn reduce_bus(bus_data: &mut BusData,  program_archive : &ProgramArchive) {
    let mut environment = Environment::new();
    for param in bus_data.get_name_of_params() {
        environment.add_variable(param, ());
    }
    let body = bus_data.get_mut_body();
    reduce_types_in_statement(body, &mut environment, program_archive);
}

fn reduce_types_in_statement(stmt: &mut Statement, environment: &mut Environment,  program_archive : &ProgramArchive) {
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
            reduce_types_in_expression(rhe, environment,program_archive);
        },
    }
}

fn reduce_types_in_log_call(args: &mut Vec<LogArgument>, environment: &Environment, program_archive : &ProgramArchive){
    for arg in args {
        if let LogArgument::LogExp(exp) = arg {
            reduce_types_in_expression(exp, environment, program_archive);
        }
    }
}

fn reduce_types_in_expression(expression: &mut Expression, environment: &Environment, program_archive : &ProgramArchive) {
    use Expression::*;
    match expression {
        Variable { name, access, meta, .. } => {
            reduce_types_in_variable(name, environment, access, meta,program_archive)
        }
        InfixOp { lhe, rhe, .. } => reduce_types_in_infix(lhe, rhe, environment,program_archive),
        PrefixOp { rhe, .. } => reduce_types_in_expression(rhe, environment,program_archive),
        ParallelOp { rhe, .. } => reduce_types_in_expression(rhe, environment,program_archive),
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            reduce_types_in_inline_switch(cond, if_true, if_false, environment,program_archive)
        }
        Call { args, .. } => reduce_types_in_vec_of_expressions(args, environment,program_archive),
        ArrayInLine { values, .. } => reduce_types_in_vec_of_expressions(values, environment,program_archive),
        UniformArray { value, dimension, .. } => {
            reduce_types_in_expression(value, environment,program_archive);
            reduce_types_in_expression(dimension, environment,program_archive);
        }
        Number(..) => {}
        BusCall { args, .. } => {
            reduce_types_in_vec_of_expressions(args, environment,program_archive);
        },
        _ => {unreachable!("Anonymous calls should not be reachable at this point."); }
    }
}

fn reduce_types_in_constraint_equality(
    lhe: &mut Expression,
    rhe: &mut Expression,
    environment: &mut Environment,
    program_archive : &ProgramArchive
) {
    reduce_types_in_expression(lhe, environment,program_archive);
    reduce_types_in_expression(rhe, environment,program_archive);
}

fn reduce_types_in_declaration(
    xtype: &VariableType,
    name: &str,
    dimensions: &mut [Expression],
    environment: &mut Environment,
    program_archive : &ProgramArchive
) {
    use VariableType::*;
    if *xtype == Var {
        environment.add_variable(name, ());
    } else if *xtype == Component || *xtype == AnonymousComponent {
        let mut typ = TypeKnowledge::default(); 
        typ.set_reduces_to(TypeReduction::Component(None));
        environment.add_component(name, typ);
    } else if let Bus(_,_,_) = *xtype{
        let mut typ = TypeKnowledge::default(); 
        typ.set_reduces_to(TypeReduction::Bus(None));
        environment.add_intermediate_bus(name, typ)
    } else {
        environment.add_intermediate(name, ());
    }
    reduce_types_in_vec_of_expressions(dimensions, environment,program_archive);
}

fn reduce_types_in_substitution(
    name: &str,
    access: &mut [Access],
    environment: &mut Environment,
    expr: &mut Expression,
    meta: &mut Meta,
    program_archive : &ProgramArchive
) {
    reduce_types_in_variable(name, environment, access, meta,program_archive);
    reduce_types_in_expression(expr, environment,program_archive);
    let mut is_simple_component_or_bus = true;
    for a in access{
        if let Access::ComponentAccess(_) = a  {
            is_simple_component_or_bus = false;
        }
    }
    if environment.has_intermediate_bus(name) && is_simple_component_or_bus {
        if let Some(xtype) = environment.get_mut_bus(name) { 
            if !xtype.is_initialized(){
                xtype.set_reduces_to(expr.get_meta().get_type_knowledge().get_reduces_to())
            }
        }
    }
    if environment.has_component(name) && is_simple_component_or_bus {
        if let Some(xtype) = environment.get_mut_component(name) { 
            if !xtype.is_initialized(){
                xtype.set_reduces_to(expr.get_meta().get_type_knowledge().get_reduces_to())
            }
        }
    }
}

fn reduce_types_in_while(
    cond: &mut Expression,
    stmt: &mut Statement,
    environment: &mut Environment,
    program_archive : &ProgramArchive
) {
    reduce_types_in_expression(cond, environment,program_archive);
    reduce_types_in_statement(stmt, environment, program_archive);
}

fn reduce_types_in_conditional(
    cond: &mut Expression,
    if_branch: &mut Statement,
    else_branch: &mut Option<Box<Statement>>,
    environment: &mut Environment,
    program_archive : &ProgramArchive
) {
    reduce_types_in_expression(cond, environment,program_archive);
    reduce_types_in_statement(if_branch, environment,program_archive);
    if let Option::Some(else_stmt) = else_branch {
        reduce_types_in_statement(else_stmt, environment,program_archive);
    }
}

fn reduce_types_in_vec_of_statements(vec: &mut [Statement], environment: &mut Environment, program_archive : &ProgramArchive) {
    for stmt in vec {
        reduce_types_in_statement(stmt, environment,program_archive);
    }
}

fn reduce_types_in_variable(
    name: &str,
    environment: &Environment,
    access: &mut [Access],
    meta: &mut Meta,
    program_archive : &ProgramArchive
) {
    use Access::*;
    use TypeReduction::*;
    let mut reduction = if environment.has_signal(name) {
        Signal
    } else if environment.has_component(name) {
        environment.get_component(name).unwrap().get_reduces_to()
    } else  if environment.has_bus(name){
        environment.get_bus(name).unwrap().get_reduces_to()
    } else {
        Variable
    };

    for acc in access {
        match acc {
            ComponentAccess(name) => {
                match reduction{
                    Variable => unreachable!(),
                    Component(ref comp) => {
                        if let Some(comp) = comp {
                            let template = program_archive.get_template_data(comp.as_str());
                            let wire = if let Some(wire) = template.get_inputs().get(name)  { wire }
                                                      else if  let Some(wire) = template.get_outputs().get(name) { wire }
                                                      else {unreachable!()};
                            match wire.get_type(){
                                    WireType::Signal => reduction = Signal,
                                    WireType::Bus(new_name) => reduction = Bus(Some(new_name)),
                             }
                        }
                    },
                    Bus(ref b) => {
                        if let Some(b) = b {
                            let busdata = program_archive.get_bus_data(b.as_str());
                                let wire = busdata.get_fields().get(name).unwrap();
                                match wire.get_type(){
                                    WireType::Signal => reduction = Signal,
                                    WireType::Bus(new_name) => reduction = Bus(Some(new_name)),
                                }
                        }
                    },
                    Signal => reduction = Tag,
                    Tag => unreachable!(),
                }
            },
            ArrayAccess(exp) => {
                reduce_types_in_expression(exp, environment,program_archive)
            },
        }
    }
    meta.get_mut_type_knowledge().set_reduces_to(reduction);
}

fn reduce_types_in_infix(lhe: &mut Expression, rhe: &mut Expression, environment: &Environment,  program_archive : &ProgramArchive) {
    reduce_types_in_expression(lhe, environment, program_archive);
    reduce_types_in_expression(rhe, environment, program_archive);
}

fn reduce_types_in_inline_switch(
    cond: &mut Expression,
    if_true: &mut Expression,
    if_false: &mut Expression,
    environment: &Environment,
    program_archive : &ProgramArchive
) {
    reduce_types_in_expression(cond, environment,program_archive);
    reduce_types_in_expression(if_true, environment,program_archive);
    reduce_types_in_expression(if_false, environment,program_archive);
}

fn reduce_types_in_vec_of_expressions(vec: &mut [Expression], environment: &Environment, program_archive : &ProgramArchive) {
    for expr in vec {
        reduce_types_in_expression(expr, environment,program_archive);
    }
}
