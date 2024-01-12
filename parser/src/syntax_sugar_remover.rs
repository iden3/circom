use program_structure::ast::*;
use program_structure::statement_builders::{build_block, build_substitution};
use program_structure::error_definition::Report;
use program_structure::expression_builders::{build_call, build_tuple, build_parallel_op};
use program_structure::file_definition::FileLibrary;
use program_structure::program_archive::ProgramArchive;
use program_structure::statement_builders::{build_declaration, build_log_call, build_initialization_block};
use program_structure::template_data::TemplateData;
use std::collections::{HashMap, BTreeMap};
use num_bigint::BigInt;



pub fn apply_syntactic_sugar(program_archive : &mut  ProgramArchive) -> Result<(), Report> {
    if program_archive.get_main_expression().is_anonymous_comp() {
        return Result::Err(anonymous_general_error(program_archive.get_main_expression().get_meta().clone(),"The main component cannot contain an anonymous call  ".to_string()));
     
    }
    let old_templates = program_archive.templates.clone();

    for (_name, t) in &mut program_archive.templates {
        let old_body = t.get_body().clone();
        check_anonymous_components_statement(&old_body)?;
        let (new_body, component_decs, variable_decs, mut substitutions) = remove_anonymous_from_statement(&old_templates, &program_archive.file_library, old_body, &None)?;
        if let Statement::Block { meta, mut stmts } = new_body {
            let mut init_block = vec![
                build_initialization_block(meta.clone(), VariableType::Var, variable_decs),
                build_initialization_block(meta.clone(), VariableType::Component, component_decs)];
            init_block.append(&mut substitutions);
            init_block.append(&mut stmts);
            let new_body_with_inits = build_block(meta, init_block);
            check_tuples_statement(&new_body_with_inits)?;
            let new_body = remove_tuples_from_statement(new_body_with_inits)?;
            t.set_body(new_body);
        } else{
            unreachable!()
        }
    }


    for (_, t) in &mut program_archive.functions {
        let old_body = t.get_body().clone();
        if old_body.contains_anonymous_comp(){
            return Result::Err(anonymous_general_error(old_body.get_meta().clone(),"Functions cannot contain calls to anonymous templates".to_string()));
        } else{
            check_tuples_statement(&old_body)?;
            let new_body = remove_tuples_from_statement(old_body)?;
            t.set_body(new_body);
        }            
    }
    Result::Ok(())
}


fn check_anonymous_components_statement(
    stm : &Statement,
) -> Result<(), Report>{
    match stm {
        Statement::MultSubstitution {meta, lhe, rhe,  op, ..} => {
            if lhe.contains_anonymous_comp() {
                Result::Err(anonymous_general_error(
                    meta.clone(),
                    "An anonymous component cannot be used in the left side of an assignment".to_string())
                )
            } else{
                if rhe.contains_anonymous_comp() && *op == AssignOp::AssignSignal{
                    let error = format!("Anonymous components only admit the use of the operator <==");
                    Result::Err(anonymous_general_error(meta.clone(),error))
                } else{
                    check_anonymous_components_expression(rhe)
                }
            }
        },
        Statement::IfThenElse { meta, cond, if_case, else_case, .. } 
        => { 
            if cond.contains_anonymous_comp() {
                Result::Err(anonymous_inside_condition_error(meta.clone()))
            } else{
                check_anonymous_components_statement(if_case)?;
                if else_case.is_some(){
                    check_anonymous_components_statement(else_case.as_ref().unwrap())?;
                }
                Result::Ok(())
            }
        }
        Statement::While { meta, cond, stmt, .. }   => {
            if cond.contains_anonymous_comp() {
                Result::Err(anonymous_inside_condition_error(meta.clone()))
            } else{
                check_anonymous_components_statement(stmt)
            }
        }     
        Statement::LogCall {meta, args } => {
            for arg in args {
                if let program_structure::ast::LogArgument::LogExp( exp ) = arg {
                    if exp.contains_anonymous_comp() {
                        return Result::Err(anonymous_general_error(meta.clone() ,"An anonymous component cannot be used inside a log".to_string()))
                    }
                }
            }
            Result::Ok(())
        }  
        Statement::Assert { meta, arg}   => {
            if arg.contains_anonymous_comp() {
                Result::Err(anonymous_general_error(meta.clone(), "An anonymous component cannot be used inside an assert".to_string()))
            } else{
                Result::Ok(())
            }
        }
        Statement::Return {  meta, value: arg}=> {
            if arg.contains_anonymous_comp(){
                Result::Err(anonymous_general_error(meta.clone(), "An anonymous component cannot be used inside a function ".to_string()))
            } else{
                Result::Ok(())
            }
        }
        Statement::ConstraintEquality {meta, lhe, rhe } => {
            if lhe.contains_anonymous_comp() || rhe.contains_anonymous_comp() {
                Result::Err(anonymous_general_error(meta.clone(), "An anonymous component cannot be used with operator === ".to_string()))
            }
            else{
                Result::Ok(()) 
            }
        }
        Statement::Declaration { meta, dimensions, .. } => {
            for exp in dimensions{
                if exp.contains_anonymous_comp(){
                    return Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()));
                }
            }
            Result::Ok(())
        }
        Statement::InitializationBlock { initializations, .. } =>
        {
            for stmt in initializations {
                check_anonymous_components_statement(stmt)?;
            }
            Result::Ok(())
        }
        Statement::Block {stmts, .. } => { 

            for stmt in stmts {
                check_anonymous_components_statement(stmt)?;
            }
            Result::Ok(())
        }
        Statement::Substitution { meta, rhe, access, op, ..} => {
            use program_structure::ast::Access::ComponentAccess;
            use program_structure::ast::Access::ArrayAccess;
            for acc in access{
                match acc{
                    ArrayAccess(exp) =>{
                        if exp.contains_anonymous_comp(){
                            return Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()));
                        }
                    },
                    ComponentAccess(_)=>{},
                }
            }

            if rhe.contains_anonymous_comp() && *op == AssignOp::AssignSignal{
                let error = format!("Anonymous components only admit the use of the operator <==");
                Result::Err(anonymous_general_error(meta.clone(),error))
            } else{
                check_anonymous_components_expression(rhe)
            }
        }
        Statement::UnderscoreSubstitution { .. } => unreachable!(),
    }
}

pub fn check_anonymous_components_expression(
    exp : &Expression,
) -> Result<(),Report>{
    use Expression::*;
    match exp {
        ArrayInLine { meta, values, .. } => {    
            for value in values{
                if value.contains_anonymous_comp() {
                    return Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()));
                }
            }
            Result::Ok(())
        }, 
        UniformArray { meta, value, dimension } => {
            if value.contains_anonymous_comp() || dimension.contains_anonymous_comp() {
                Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()))
            } else{
                Result::Ok(())
            }
        },
        Number(_, _) => { 
            Result::Ok(()) 
        },
        Variable { meta, access, .. } => {
            use program_structure::ast::Access::ComponentAccess;
            use program_structure::ast::Access::ArrayAccess;
            for acc in access{
                match acc{
                    ArrayAccess(exp) =>{
                        if exp.contains_anonymous_comp(){
                            return Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()));
                        }
                    },
                    ComponentAccess(_)=>{},
                }
            }
            Result::Ok(())
        },
        InfixOp { meta, lhe, rhe, .. } => {
            if lhe.contains_anonymous_comp() || rhe.contains_anonymous_comp() {
                Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used in the middle of an operation ".to_string()))
            } else{
                Result::Ok(())
            }
        },
        PrefixOp { meta, rhe, .. } => {
            if rhe.contains_anonymous_comp()  {
                Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used in the middle of an operation ".to_string()))
            } else{
                Result::Ok(())
            }
        },
        InlineSwitchOp { meta, cond, if_true,  if_false } => {
            if cond.contains_anonymous_comp() || if_true.contains_anonymous_comp() || if_false.contains_anonymous_comp() {
                Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used inside an inline switch ".to_string()))
            } else{
                Result::Ok(())
            }
        },
        Call { meta, args, .. } => {
            for value in args{
                if value.contains_anonymous_comp() {
                    return Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used as a parameter in a template call ".to_string()));
                }
            }
            Result::Ok(())
        },
        AnonymousComp {meta, params, signals, .. } => {
            for value in params{
                if value.contains_anonymous_comp() {
                    return Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used as a parameter in a template call ".to_string()));
                }
            }
            for value in signals{
                check_anonymous_components_expression(value)?;
            }
            Result::Ok(())
        },
        Tuple {values, .. } => {

            for val in values{
                check_anonymous_components_expression(val)?;
            }
            Result::Ok(())
        },
        ParallelOp { meta, rhe } => {
            if !rhe.is_call() && !rhe.is_anonymous_comp() && rhe.contains_anonymous_comp() {
                return Result::Err(anonymous_general_error(meta.clone(),"Bad use of parallel operator in combination with anonymous components ".to_string()));       
            }
            else if rhe.is_call() && rhe.contains_anonymous_comp() {
                return Result::Err(anonymous_general_error(meta.clone(),"An anonymous component cannot be used as a parameter in a template call ".to_string()));
            }
            Result::Ok(())
        },
    }
}


// (Body, init_components, init_variables, substitutions)
pub type UpdatedStatement = (Statement, Vec<Statement>, Vec<Statement>, Vec<Statement>);

// (init_components, substitutions, expression)
pub type UpdatedExpression = (Vec<Statement>, Vec<Statement>, Expression);

fn remove_anonymous_from_statement(
    templates : &HashMap<String, TemplateData>, 
    file_lib : &FileLibrary,  
    stm : Statement,
    var_access: &Option<Expression>
) -> Result< UpdatedStatement, Report>{
    match stm {
        Statement::MultSubstitution { meta, lhe, op, rhe } => {

            let (comp_declarations, mut substitutions, new_rhe) = remove_anonymous_from_expression(templates, file_lib, rhe, var_access)?;
            let subs = Statement::MultSubstitution { meta: meta.clone(), lhe: lhe, op: op, rhe: new_rhe };
            if substitutions.is_empty(){
                Result::Ok((subs, comp_declarations, Vec::new(), Vec::new()))
            }else{
                substitutions.push(subs);
                Result::Ok((Statement::Block { meta : meta, stmts : substitutions}, comp_declarations, Vec::new(), Vec::new()))   
            }
        },
        Statement::IfThenElse { meta, cond, if_case, else_case } 
        => { 

            let (if_body, mut if_comp_dec, mut if_var_dec, mut if_subs) = remove_anonymous_from_statement(templates, file_lib, *if_case, var_access)?;
            let b_if = Box::new(if_body);
            if else_case.is_none(){
                Result::Ok((Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::None}, if_comp_dec, if_var_dec, if_subs))
            }else {
                let else_c = *(else_case.unwrap());
                let (else_body, mut else_comp_dec, mut else_var_dec, mut else_subs) = remove_anonymous_from_statement(templates, file_lib, else_c, var_access)?;
                let b_else = Box::new(else_body);
                if_comp_dec.append(&mut else_comp_dec);
                if_var_dec.append(&mut else_var_dec);
                if_subs.append(&mut else_subs);
                Result::Ok((Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::Some(b_else)}, if_comp_dec, if_var_dec, if_subs))
            }
        }
        Statement::While { meta, cond, stmt }   => {
            let id_var_while = "anon_var_".to_string() + &file_lib.get_line(meta.start, meta.get_file_id()).unwrap().to_string() + "_" + &meta.start.to_string();
            let var_access = Expression::Variable{meta: meta.clone(), name: id_var_while.clone(), access: Vec::new()};
            let mut var_declarations = vec![];
            let mut subs_out = vec![];
            let (body, comp_dec, mut var_dec, mut subs) = remove_anonymous_from_statement(templates, file_lib, *stmt, &Some(var_access.clone()))?;
            let b_while = if !comp_dec.is_empty(){
                var_declarations.push(
                    build_declaration(
                        meta.clone(), 
                        VariableType::Var, 
                        id_var_while.clone(),
                        Vec::new(),
                    )
                );
                subs.push(
                    build_substitution(
                        meta.clone(), 
                        id_var_while.clone(), 
                        vec![], 
                        AssignOp::AssignVar, 
                        Expression::Number(meta.clone(), BigInt::from(0))
                    )
                );
                var_declarations.append(&mut var_dec);
                subs_out.append(&mut subs);
                let next_access = Expression::InfixOp{
                    meta: meta.clone(),
                    infix_op: ExpressionInfixOpcode::Add,
                    lhe: Box::new(var_access),
                    rhe: Box::new(Expression::Number(meta.clone(),  BigInt::from(1))),
                };
                let subs_access = Statement::Substitution{
                    meta: meta.clone(),
                    var: id_var_while,
                    access: Vec::new(),
                    op: AssignOp::AssignVar,
                    rhe: next_access,
                };
                    
                let new_block = Statement::Block{
                    meta: meta.clone(),
                    stmts: vec![body, subs_access],
                };
                Box::new(new_block)
            } else{
                Box::new(body)
            };
            Result::Ok((Statement::While { meta: meta, cond: cond, stmt: b_while}, comp_dec, var_declarations, subs_out))
        },     

        Statement::InitializationBlock { meta, xtype, initializations } =>
        {
            let mut new_inits = Vec::new();
            let mut comp_inits = Vec::new();
            let mut var_inits = Vec::new();
            let mut subs = Vec::new();

            for stmt in initializations {
                let (stmt_ok, mut comps, mut vars, mut sub) = remove_anonymous_from_statement(templates, file_lib, stmt, var_access)?;
                new_inits.push(stmt_ok);
                comp_inits.append(&mut comps);
                var_inits.append(&mut vars);
                subs.append(&mut sub);
            }
            Result::Ok((Statement::InitializationBlock { meta: meta, xtype: xtype, initializations: new_inits }, comp_inits, var_inits, subs))
        }
        Statement::Block { meta, stmts } => { 
            let mut new_stmts = Vec::new();
            let mut comp_inits = Vec::new();
            let mut var_inits = Vec::new();
            let mut subs = Vec::new();
            for stmt in stmts {
                let (stmt_ok, mut comps, mut vars, mut sub) = remove_anonymous_from_statement(templates, file_lib, stmt, var_access)?;
                new_stmts.push(stmt_ok);
                comp_inits.append(&mut comps);
                var_inits.append(&mut vars);
                subs.append(&mut sub);
            }
            Result::Ok((Statement::Block { meta, stmts: new_stmts}, comp_inits, var_inits, subs))
        }
        Statement::Substitution {  meta, var, op, rhe, access} => {
            let (comp_declarations, mut stmts, new_rhe) = remove_anonymous_from_expression(templates, file_lib, rhe, var_access)?;
            let subs = Statement::Substitution { meta: meta.clone(), var: var, access: access, op: op, rhe: new_rhe };
            if stmts.is_empty(){
                Result::Ok((subs, comp_declarations, Vec::new(), Vec::new()))
            }else{
                stmts.push(subs);
                Result::Ok((Statement::Block { meta, stmts}, comp_declarations, Vec::new(), Vec::new()))   
            }
        }
        Statement::UnderscoreSubstitution { .. } => unreachable!(),
        _ => {
            Result::Ok((stm, Vec::new(), Vec::new(), Vec::new()))
        }
    }
}

// returns a block with the component declarations, the substitutions and finally the output expression
pub fn remove_anonymous_from_expression(
    templates : &HashMap<String, TemplateData>, 
    file_lib : &FileLibrary,
    exp : Expression,
    var_access: &Option<Expression>, // in case the call is inside a loop, variable used to control the access
) -> Result<UpdatedExpression,Report>{
    use Expression::*;
    match exp {
        AnonymousComp { meta, id, params, signals, names,  is_parallel } => {
            let mut declarations = Vec::new();
            let mut seq_substs = Vec::new();

            // get the template we are calling to
            let template = templates.get(&id);
            if template.is_none(){
                return Result::Err(anonymous_general_error(meta.clone(),format!("The template {} does not exist", id)));
            }
            let id_anon_temp = id.to_string() + "_" + &file_lib.get_line(meta.start, meta.get_file_id()).unwrap().to_string() + "_" + &meta.start.to_string();
            
            // in case we are not inside a loop, we can automatically convert into a component
            if var_access.is_none(){
                declarations.push(build_declaration(
                    meta.clone(), 
                    VariableType::Component, 
                    id_anon_temp.clone(),
                    Vec::new(),
                ));
            } else{ // we generate an anonymous component, it depends on the var_access indicating the loop
                declarations.push(build_declaration(
                    meta.clone(), 
                    VariableType::AnonymousComponent, 
                    id_anon_temp.clone(),
                    vec![var_access.as_ref().unwrap().clone()],
                ));
            }

            // build the call generating the component
            let call = build_call(meta.clone(), id.clone(), params.clone());
            let exp_with_call = if is_parallel {
                build_parallel_op(meta.clone(), call)
            } else {  
                call
            };
            // in case we are in a loop in only generates a position, needs the var_access reference
            let access = if var_access.is_none(){
                 Vec::new()
            } else{
                vec![build_array_access(var_access.as_ref().unwrap().clone())]
            };
            // in loop: id_anon_temp[var_access] = (parallel) Template(params);
            // out loop: id_anon_temp = (parallel) Template(params)
            let sub = build_substitution(
                meta.clone(), 
                id_anon_temp.clone(), 
                access.clone(), 
                AssignOp::AssignVar, 
                exp_with_call
            );
            seq_substs.push(sub);

            // assign the inputs
            // reorder the signals in new_signals (case names)
            let mut inputs_to_assignments = BTreeMap::new();

            if let Some(m) = names { // in case we have a list of names and assignments
                let inputs = template.unwrap().get_inputs();
                let mut n_expr = 0;
                for (operator, name) in m{
                    if operator != AssignOp::AssignConstraintSignal{
                        let error = format!("Anonymous components only admit the use of the operator <==");
                        return Result::Err(anonymous_general_error(meta.clone(),error));
                    }
                    if inputs.contains_key(&name){
                        inputs_to_assignments.insert(name, (operator, signals[n_expr].clone()));
                    } else{
                        let error = format!("The template {} does not have an input signal named {}", template.unwrap().get_name(),  name);
                        return Result::Err(anonymous_general_error(meta.clone(),error));
                    }
                    n_expr += 1;
                }   
                if inputs.len() != inputs_to_assignments.len() {
                    return Result::Err(anonymous_general_error(meta.clone(),"The number of template input signals must coincide with the number of input parameters ".to_string()));
                }            
            }
            else{
                let inputs = template.unwrap().get_declaration_inputs();
                let mut n_expr = 0;
                for value in signals {
                    inputs_to_assignments.insert(inputs[n_expr].0.clone(), (AssignOp::AssignConstraintSignal, value));
                    n_expr += 1;
                }
                
                if inputs.len() != inputs_to_assignments.len() {
                    return Result::Err(anonymous_general_error(meta.clone(),"The number of template input signals must coincide with the number of input parameters ".to_string()));
                }
            }
            

            // generate the substitutions for the inputs
            for (name_signal, (operator, expr)) in inputs_to_assignments{
                let mut acc = if var_access.is_none(){
                    Vec::new()
                } else{
                    vec![build_array_access(var_access.as_ref().unwrap().clone())]
                };
                acc.push(Access::ComponentAccess(name_signal.clone()));
                let  (mut declarations2, mut stmts,new_exp) = remove_anonymous_from_expression(
                        templates, 
                        file_lib, 
                        expr,
                        var_access
                )?;
 
                seq_substs.append(&mut stmts);
                declarations.append(&mut declarations2);
                let subs = Statement::Substitution { meta: meta.clone(), var: id_anon_temp.clone(), access: acc, op: operator, rhe: new_exp };
                seq_substs.push(subs);
            }
            // generate the expression for the outputs -> return as expression (if single out) or tuple
            let outputs = template.unwrap().get_declaration_outputs();
            if outputs.len() == 1 {
                let output = outputs.get(0).unwrap().0.clone();
                let mut acc = if var_access.is_none(){
                    Vec::new()
                } else{
                    vec![build_array_access(var_access.as_ref().unwrap().clone())]
                };

                acc.push(Access::ComponentAccess(output.clone()));
                let out_exp = Expression::Variable { meta: meta.clone(), name: id_anon_temp, access: acc };
                Result::Ok((declarations, vec![Statement::Block { meta: meta.clone(), stmts: seq_substs }], out_exp))

             } else{
                let mut new_values = Vec::new(); 
                for output in outputs {
                    let mut acc = if var_access.is_none(){
                        Vec::new()
                    } else{
                        vec![build_array_access(var_access.as_ref().unwrap().clone())]
                    };
                    acc.push(Access::ComponentAccess(output.0.clone()));
                    let out_exp = Expression::Variable { meta: meta.clone(), name: id_anon_temp.clone(), access: acc };
                    new_values.push(out_exp);
                }
                let out_exp = Tuple {meta : meta.clone(), values : new_values};
                Result::Ok((declarations, vec![Statement::Block { meta: meta.clone(), stmts: seq_substs }], out_exp))

            }
        },
        Tuple { meta, values } => {
            let mut new_values = Vec::new();
            let mut new_stmts : Vec<Statement> = Vec::new();
            let mut declarations : Vec<Statement> = Vec::new();
            for val in values{
                let result = remove_anonymous_from_expression(templates, file_lib, val, var_access);
                match result {
                    Ok((mut declaration, mut stm, val2)) => {
                        new_stmts.append(&mut stm);
                        new_values.push(val2);
                        declarations.append(&mut declaration);
                    },
                    Err(er) => {return Result::Err(er);},
                }
            }
            Result::Ok((declarations, new_stmts, build_tuple(meta.clone(), new_values)))
        },
        ParallelOp { meta, rhe } => {
            if rhe.is_anonymous_comp(){
                let rhe2 = rhe.make_anonymous_parallel();
                remove_anonymous_from_expression(templates, file_lib, rhe2, var_access)
            } else{
                Result::Ok((Vec::new(),Vec::new(), ParallelOp { meta, rhe }))
            }
        },
        _ =>{
            Result::Ok((Vec::new(),Vec::new(),exp))
        }
    }
}


fn check_tuples_statement(stm: &Statement)-> Result<(), Report>{
    match stm{
        Statement::MultSubstitution { lhe, rhe, ..  } => {
            check_tuples_expression(lhe)?;
            check_tuples_expression(rhe)?;
            Result::Ok(())
        },
        Statement::IfThenElse { cond, if_case, else_case, meta, .. } 
        => { 
            if cond.contains_tuple() {
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used inside a condition ".to_string()))     
            } else{
                check_tuples_statement(if_case)?;
                if else_case.is_some(){
                    check_tuples_statement(else_case.as_ref().unwrap())?;
                }
                Result::Ok(())
            }
        }

        Statement::While { meta, cond, stmt }   => {
            if cond.contains_tuple() {
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used inside a condition ".to_string()))       
           } else{      
                check_tuples_statement(stmt)
            }
        }     
        Statement::LogCall {args, .. } => {
            for arg in args {
                match arg {
                    LogArgument::LogStr(_) => {},
                    LogArgument::LogExp(exp) => {
                        check_tuples_expression(&exp)?;
                    },
                }
            }
            Result::Ok(())
        }  
        Statement::Assert { meta, arg}   => { 
            if arg.contains_tuple(){
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used in a return ".to_string()))       
            }
            else{ 
                Result::Ok(())
            }
        }
        Statement::Return {  meta, value: arg}=> {
            if arg.contains_tuple(){
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used inside a function ".to_string()))     
            }
            else{ 
                Result::Ok(())
            }
        }
        Statement::ConstraintEquality {meta, lhe, rhe } => {
            if lhe.contains_tuple() || rhe.contains_tuple() {
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used with the operator === ".to_string()))       
            }
            else{ 
                Result::Ok(()) 
            }
        }
        Statement::Declaration { meta,
                                 dimensions, .. } =>
        {
            for exp in dimensions.clone(){
                if exp.contains_tuple(){
                    return Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array ".to_string()));       
                }
            }
            Result::Ok(())
        }
        Statement::InitializationBlock {initializations, ..} =>
        {
            for stmt in initializations {
                check_tuples_statement(stmt)?;
            }
            Result::Ok(())
        }
        Statement::Block { stmts, ..} => { 
            for stmt in stmts {
                check_tuples_statement(stmt)?;
            }
            Result::Ok(())
        }
        Statement::Substitution { rhe, access, meta,  ..} => {
            use program_structure::ast::Access::ComponentAccess;
            use program_structure::ast::Access::ArrayAccess;
            for acc in access{
                match acc{
                    ArrayAccess(exp) =>{
                        if exp.contains_tuple(){
                            return Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array".to_string()));
                        }
                    },
                    ComponentAccess(_)=>{},
                }
            }
            check_tuples_expression(rhe)
        }
        Statement::UnderscoreSubstitution { .. } => unreachable!(),
    }
}


pub fn check_tuples_expression(exp: &Expression) -> Result<(), Report>{
    use Expression::*;
    match exp{
        ArrayInLine { meta, values } => {    
            for value in values{
                if value.contains_tuple() {
                    return Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array ".to_string()));       
                }
            }
            Result::Ok(())
        }, 
        UniformArray { meta, value, dimension } => {
            if value.contains_tuple() || dimension.contains_tuple() {
                return Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array ".to_string()));       
            }
            Result::Ok(())
        },
        Number(_, _) => {
            Result::Ok(())
        },
        Variable { access, meta,  .. } => {
            use program_structure::ast::Access::*;
            for acc in access{
                match acc{
                    ArrayAccess(exp) =>{
                        if exp.contains_tuple(){
                            return Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array".to_string()));
                        }
                    },
                    ComponentAccess(_)=>{},
                }
            }
            Result::Ok(())
        },
        InfixOp { meta, lhe, rhe, .. } => {
            if lhe.contains_tuple() || rhe.contains_tuple() {
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used in the middle of an operation".to_string()))     
            } else{
                Result::Ok(())
            }
        },
        PrefixOp { meta, rhe, .. } => {
            if rhe.contains_tuple()  {
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used in the middle of an operation".to_string()))     
            } else{
                Result::Ok(())
            }
        },
        InlineSwitchOp { meta, cond, if_true,  if_false } => {
            if cond.contains_tuple() || if_true.contains_tuple() || if_false.contains_tuple() {
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used inside an inline switch".to_string()))      
            } else{
                Result::Ok(())
            }
        },
        Call { meta, args, .. } => {
            for value in args{
                if value.contains_tuple() {
                    return Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used as a parameter of a function call".to_string()));       
                }
            }
            Result::Ok(())
        },
        AnonymousComp { .. } => {
            unreachable!();
        }
        Tuple { values, .. } => {
            for val in values {
                check_tuples_expression(val)?;                          
            }
            Result::Ok(())
        },
        ParallelOp { meta, rhe} => {
            if rhe.contains_tuple()  {
                Result::Err(tuple_general_error(meta.clone(),"A tuple cannot be used in a parallel operator ".to_string()))       
            } else{
                Result::Ok(())
            }
        },
    }
}

fn remove_tuples_from_statement(stm: Statement) -> Result<Statement, Report> {
    match stm{
        Statement::MultSubstitution { meta, lhe, op, rhe  } => {
            let new_exp_lhe = remove_tuple_from_expression(lhe);
            let new_exp_rhe = remove_tuple_from_expression(rhe);
            match (new_exp_lhe, new_exp_rhe) {
                (Expression::Tuple { values: mut values1, .. },
                    Expression::Tuple { values: mut values2, .. }) => {
                    if values1.len() == values2.len() {
                        let mut substs = Vec::new();
                        while  values1.len() > 0 {
                            let lhe = values1.remove(0);
                            if let Expression::Variable { meta, name, access } = lhe {  
                                let rhe = values2.remove(0);
                                if name != "_" {                                
                                    substs.push(build_substitution(meta, name, access, op, rhe));
                                } else{
                                    substs.push(Statement::UnderscoreSubstitution { meta: meta, op, rhe: rhe });
                                }
                            } else{   
                                return Result::Err(tuple_general_error(meta.clone(),"The elements of the receiving tuple must be signals or variables.".to_string()));
                            }
                        }
                        return Result::Ok(build_block(meta.clone(),substs));
                    } else if values1.len() > 0 {
                        return Result::Err(tuple_general_error(meta.clone(),"The number of elements in both tuples does not coincide".to_string()));           
                    } else {
                        return Result::Err(tuple_general_error(meta.clone(),"This expression must be in the right side of an assignment".to_string()));           
                    }
                },
                (lhe, rhe) => { 
                    if lhe.is_tuple() || lhe.is_variable(){
                        return Result::Err(tuple_general_error(rhe.get_meta().clone(),"This expression must be a tuple or an anonymous component".to_string()));
                    } else {
                        return Result::Err(tuple_general_error(lhe.get_meta().clone(),"This expression must be a tuple, a component, a signal or a variable ".to_string()));    
                    }
                }
            }
        },
        Statement::IfThenElse { meta, cond, if_case, else_case } 
        => { 
            let if_ok = remove_tuples_from_statement(*if_case)?;
            let b_if = Box::new(if_ok);
            if else_case.is_none(){
                Result::Ok(Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::None})
            }else {
                let else_c = *(else_case.unwrap());
                let else_ok = remove_tuples_from_statement(else_c)?;
                let b_else = Box::new(else_ok);
                Result::Ok(Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::Some(b_else)})
            }
        }

        Statement::While { meta, cond, stmt }   => {
            let while_ok = remove_tuples_from_statement(*stmt)?;
            let b_while = Box::new(while_ok);
            Result::Ok(Statement::While { meta : meta, cond : cond, stmt : b_while})
        }     
        Statement::LogCall {meta, args } => {
            let mut newargs = Vec::new();
            for arg in args {
                match arg {
                    LogArgument::LogStr(str) => {
                        newargs.push(LogArgument::LogStr(str));
                    },
                    LogArgument::LogExp(exp) => {
                        let mut args2 = separate_tuple_for_logcall(vec![exp]);
                        newargs.append(&mut args2);
                    },
                }
            }
            Result::Ok(build_log_call(meta, newargs))
        }  
        Statement::InitializationBlock { meta, xtype, initializations } =>
        {
            let mut new_inits = Vec::new();
            for stmt in initializations {
                let stmt_ok = remove_tuples_from_statement(stmt)?;
                new_inits.push(stmt_ok);
            }
            Result::Ok(Statement::InitializationBlock { meta: meta, xtype: xtype, initializations: new_inits })
        }
        Statement::Block { meta, stmts } => { 
            let mut new_stmts = Vec::new();
            for stmt in stmts {
                let stmt_ok = remove_tuples_from_statement(stmt)?;
                new_stmts.push(stmt_ok);
            }
            Result::Ok(Statement::Block { meta : meta, stmts: new_stmts})
        }
        Statement::Substitution {  meta, var, op, rhe, access} => {
            let new_rhe = remove_tuple_from_expression(rhe);
            if new_rhe.is_tuple() {
                return Result::Err(tuple_general_error(meta.clone(),"Left-side of the statement is not a tuple".to_string()));       
            }
            if var != "_" {   
                Result::Ok(Statement::Substitution { meta: meta.clone(), var: var, access: access, op: op, rhe: new_rhe })
            }
            else {
                Result::Ok(Statement::UnderscoreSubstitution { meta: meta, op, rhe: new_rhe })
            }
        }
        Statement::UnderscoreSubstitution { .. } => unreachable!(),
        _ => Result::Ok(stm), // The rest of cases do not change the stmt (cannot contain tuples)
    }
}

fn separate_tuple_for_logcall(values: Vec<Expression>) ->  Vec<LogArgument> {
    let mut new_values = Vec::new();
    for val in values {
        if let Expression::Tuple {  values : values2, .. } = val {
            new_values.push(LogArgument::LogStr("(".to_string()));
            let mut new_values2 = separate_tuple_for_logcall(values2);
            new_values.append(&mut new_values2);
            new_values.push(LogArgument::LogStr(")".to_string()));
        }
        else {
            new_values.push(LogArgument::LogExp(val));
        }
    }
    new_values
}


pub fn remove_tuple_from_expression(exp : Expression) -> Expression{
    use Expression::*;
    match exp {
        AnonymousComp { .. } => {
            unreachable!();
        }
        Tuple { meta, values } => {
            let mut unfolded_values =  Vec::new();
            for val in values {
                let exp = remove_tuple_from_expression(val);
                if let Tuple { values: mut values2, ..} = exp {
                    unfolded_values.append(&mut values2);
                }  else {
                    unfolded_values.push(exp);
                }                               
            }
            build_tuple(meta, unfolded_values)
        },
        _ => exp,
    }
}

