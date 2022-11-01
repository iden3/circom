use program_structure::ast::*;
use program_structure::statement_builders::{build_block, build_substitution};
use program_structure::error_definition::{Report};
use program_structure::expression_builders::{build_call, build_tuple, build_parallel_op};
use program_structure::file_definition::FileLibrary;
use program_structure::program_archive::ProgramArchive;
use program_structure::statement_builders::{build_declaration, build_log_call, build_assert, build_return, build_constraint_equality, build_initialization_block};
use program_structure::template_data::{TemplateData};
use std::collections::HashMap;
use num_bigint::BigInt;



use crate::errors::{AnonymousCompError,TupleError};



fn remove_anonymous_from_statement(
    templates : &mut HashMap<String, TemplateData>, 
    file_lib : &FileLibrary,  
    stm : Statement,
    var_access: &Option<Expression>
) -> Result<(Statement,Vec<Statement>),Report>{
    match stm.clone() {
        Statement::MultSubstitution { meta, lhe, op, rhe } => {
            if lhe.contains_anonymous_comp() {
                return Result::Err(AnonymousCompError::anonymous_general_error(lhe.get_meta().clone(),"An anonymous component cannot be used in the left side of an assignment".to_string()));
                } else{
                let (mut stmts, declarations, new_rhe) = remove_anonymous_from_expression(templates, file_lib, rhe, var_access)?;
                let subs = Statement::MultSubstitution { meta: meta.clone(), lhe: lhe, op: op, rhe: new_rhe };
                let mut substs = Vec::new(); 
                if stmts.is_empty(){
                    Result::Ok((subs, declarations))
                }else{
                    substs.append(&mut stmts);
                    substs.push(subs);
                    Result::Ok((Statement::Block { meta : meta, stmts : substs}, declarations))   
                }
            }
        },
        Statement::IfThenElse { meta, cond, if_case, else_case } 
        => { 
            if cond.contains_anonymous_comp() {
                Result::Err(AnonymousCompError::anonymous_inside_condition_error(cond.get_meta().clone()))
            } else{
                let (if_ok,mut declarations) = remove_anonymous_from_statement(templates, file_lib, *if_case, var_access)?;
                let b_if = Box::new(if_ok);
                if else_case.is_none(){
                    Result::Ok((Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::None},declarations))
                }else {
                    let else_c = *(else_case.unwrap());
                    let (else_ok, mut declarations2) = remove_anonymous_from_statement(templates, file_lib, else_c, var_access)?;
                    let b_else = Box::new(else_ok);
                    declarations.append(&mut declarations2);
                    Result::Ok((Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::Some(b_else)},declarations))
                }
            }
        }
        Statement::While { meta, cond, stmt }   => {
            if cond.contains_anonymous_comp() {
                Result::Err(AnonymousCompError::anonymous_inside_condition_error(cond.get_meta().clone()))
            } else{

                let id_var_while = "anon_var_".to_string() + &file_lib.get_line(meta.start, meta.get_file_id()).unwrap().to_string() + "_" + &meta.start.to_string();
                let var_access = Expression::Variable{meta: meta.clone(), name: id_var_while.clone(), access: Vec::new()};
                let mut declarations = vec![];
                let (while_ok, mut declarations2) = remove_anonymous_from_statement(templates, file_lib, *stmt, &Some(var_access.clone()))?;
                let b_while = if !declarations2.is_empty(){
                    declarations.push(
                        build_declaration(
                            meta.clone(), 
                            VariableType::Var, 
                            id_var_while.clone(),
                            Vec::new(),
                        )
                    );
                    declarations.push(
                        build_substitution(
                            meta.clone(), 
                            id_var_while.clone(), 
                            vec![], 
                            AssignOp::AssignVar, 
                            Expression::Number(meta.clone(), BigInt::from(0))
                        )
                    );
                    declarations.append(&mut declarations2);
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
                        stmts: vec![while_ok, subs_access],
                    };
                    Box::new(new_block)
                } else{
                    Box::new(while_ok)
                };

                Result::Ok((Statement::While { meta: meta, cond: cond, stmt: b_while}, declarations))
            }
        }     
        Statement::LogCall {meta, args } => {
            for arg in &args {
                if let program_structure::ast::LogArgument::LogExp( exp ) = arg {
                    if exp.contains_anonymous_comp() {
                        return Result::Err(AnonymousCompError::anonymous_general_error(meta,"An anonymous component cannot be used inside a log".to_string()))
                    }
                }
            }
            Result::Ok((build_log_call(meta, args),Vec::new()))
        }  
        Statement::Assert { meta, arg}   => { Result::Ok((build_assert(meta, arg),Vec::new()))}
        Statement::Return {  meta, value: arg}=> {
            if arg.contains_anonymous_comp(){
                Result::Err(AnonymousCompError::anonymous_general_error(meta,"An anonymous component cannot be used inside a function ".to_string()))
            }
            else{ Result::Ok((build_return(meta, arg),Vec::new()))}
        }
        Statement::ConstraintEquality {meta, lhe, rhe } => {
            if lhe.contains_anonymous_comp() || rhe.contains_anonymous_comp() {
                Result::Err(AnonymousCompError::anonymous_general_error(meta,"An anonymous component cannot be used with operator === ".to_string()))
            }
            else{ Result::Ok((build_constraint_equality(meta, lhe, rhe),Vec::new())) }
        }
        Statement::Declaration { meta , xtype , name ,
                                 dimensions, .. } => {
            for exp in dimensions.clone(){
                if exp.contains_anonymous_comp(){
                    return Result::Err(AnonymousCompError::anonymous_general_error(exp.get_meta().clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()));
                }
            }
            Result::Ok((build_declaration(meta, xtype, name, dimensions),Vec::new()))
        }
        Statement::InitializationBlock { meta, xtype, initializations } =>
        {
            let mut new_inits = Vec::new();
            let mut declarations = Vec::new();
            for stmt in initializations {
                let (stmt_ok, mut declaration) = remove_anonymous_from_statement(templates, file_lib, stmt, var_access)?;
                new_inits.push(stmt_ok);
                declarations.append(&mut declaration)
            }
            Result::Ok((Statement::InitializationBlock { meta: meta, xtype: xtype, initializations: new_inits }, declarations))
        }
        Statement::Block { meta, stmts } => { 
            let mut new_stmts = Vec::new();
            let mut declarations  = Vec::new();
            for stmt in stmts {
                let (stmt_ok, mut declaration) = remove_anonymous_from_statement(templates, file_lib, stmt, var_access)?;
                new_stmts.push(stmt_ok);
                declarations.append(&mut declaration);
            }
            Result::Ok((Statement::Block { meta : meta, stmts: new_stmts},declarations))
        }
        Statement::Substitution {  meta, var, op, rhe, access} => {
            let (mut stmts, declarations, new_rhe) = remove_anonymous_from_expression(templates, file_lib, rhe, var_access)?;
            let subs = Statement::Substitution { meta: meta.clone(), var: var, access: access, op: op, rhe: new_rhe };
            let mut substs = Vec::new(); 
            if stmts.is_empty(){
                Result::Ok((subs, declarations))
            }else{
                substs.append(&mut stmts);
                substs.push(subs);
                Result::Ok((Statement::Block { meta : meta, stmts : substs}, declarations))   
            }
        }
    }
}

// returns a block with the substitutions, the declarations and finally the output expression
pub fn remove_anonymous_from_expression(
    templates : &mut HashMap<String, TemplateData>, 
    file_lib : & FileLibrary,
    exp : Expression,
    var_access: &Option<Expression>, // in case the call is inside a loop, variable used to control the access
) -> Result<(Vec<Statement>, Vec<Statement>, Expression),Report>{
    use Expression::*;
    match exp.clone() {
        ArrayInLine { values, .. } => {    
        for value in values{
            if value.contains_anonymous_comp() {
                return Result::Err(AnonymousCompError::anonymous_general_error(value.get_meta().clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()));
            }
        }
        Result::Ok((Vec::new(),Vec::new(),exp))
        }, 
        UniformArray { meta, value, dimension } => {
            if value.contains_anonymous_comp() || dimension.contains_anonymous_comp() {
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used to define a dimension of an array".to_string()));
            }
            Result::Ok((Vec::new(),Vec::new(),exp))
        },
        Number(_, _) => { Result::Ok((Vec::new(),Vec::new(),exp)) },
        Variable { meta, .. } => {
            if exp.contains_anonymous_comp(){
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used to access an array".to_string()));
            }
            Result::Ok((Vec::new(),Vec::new(),exp))
        },
        InfixOp { meta, lhe, rhe, .. } => {
            if lhe.contains_anonymous_comp() || rhe.contains_anonymous_comp() {
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used in the middle of an operation ".to_string()));
            }
            Result::Ok((Vec::new(),Vec::new(),exp))
        },
        PrefixOp { meta, rhe, .. } => {
            if rhe.contains_anonymous_comp()  {
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used in the middle of an operation ".to_string()));
            }
            Result::Ok((Vec::new(),Vec::new(),exp))
        },
        InlineSwitchOp { meta, cond, if_true,  if_false } => {
            if cond.contains_anonymous_comp() || if_true.contains_anonymous_comp() || if_false.contains_anonymous_comp() {
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used inside an inline switch ".to_string()));
             }
            Result::Ok((Vec::new(),Vec::new(),exp))
            /* This code is useful if we want to allow anonymous components inside InLineSwitch.
            let result_if = remove_anonymous_from_expression( templates, file_lib,     *if_true);
            let result_else = remove_anonymous_from_expression( templates, file_lib,   *if_false);
            if result_if.is_err() { Result::Err(result_if.err().unwrap())}
            else if result_else.is_err() {  Result::Err(result_else.err().unwrap())}
            else {
                let (result_if2,exp_if) = result_if.ok().unwrap();
                let (result_else2, exp_else) = result_else.ok().unwrap();
                let block_if = if result_if2.is_none() { build_block(meta.clone(), Vec::new())} else { result_if2.unwrap()};
            
                Result::Ok((Option::Some(build_conditional_block(meta.clone(), *cond.clone(), block_if, result_else2)),
                            build_inline_switch_op(meta.clone(), *cond.clone(), exp_if, exp_else)))*/

        },
        Call { meta, args, .. } => {
            for value in args{
                if value.contains_anonymous_comp() {
                    return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used as a parameter in a template call ".to_string()));
                }
            }
            Result::Ok((Vec::new(),Vec::new(),exp))
        },
        AnonymousComp { meta, id, params, signals, names,  is_parallel } => {
            let template = templates.get(&id);
            let mut declarations = Vec::new();
            if template.is_none(){
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"The template does not exist ".to_string()));
            }
            let mut i = 0;
            let mut seq_substs = Vec::new();
            let id_anon_temp = id.to_string() + "_" + &file_lib.get_line(meta.start, meta.get_file_id()).unwrap().to_string() + "_" + &meta.start.to_string();
            if var_access.is_none(){
                declarations.push(build_declaration(
                    meta.clone(), 
                    VariableType::Component, 
                    id_anon_temp.clone(),
                    Vec::new(),
                ));
            } else{
                declarations.push(build_declaration(
                    meta.clone(), 
                    VariableType::AnonymousComponent, 
                    id_anon_temp.clone(),
                    vec![var_access.as_ref().unwrap().clone()],
                ));
            }
            let call = build_call(meta.clone(), id, params);
            if call.contains_anonymous_comp(){
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used as a parameter in a template call ".to_string()));
             }

            let exp_with_call = if is_parallel {
                build_parallel_op(meta.clone(), call)
            } else {
                call
            };
            let access = if var_access.is_none(){
                 Vec::new()
            } else{
                vec![build_array_access(var_access.as_ref().unwrap().clone())]
            };
            let sub = build_substitution(meta.clone(), id_anon_temp.clone(), 
            access.clone(), AssignOp::AssignVar, exp_with_call);
            seq_substs.push(sub);
            let inputs = template.unwrap().get_declaration_inputs();
            let mut new_signals = Vec::new();
            let mut new_operators = Vec::new();
            if let Some(m) = names {
                let (operators, names) : (Vec<AssignOp>, Vec<String>) = m.iter().cloned().unzip();
                for inp in inputs{
                    if !names.contains(&inp.0) {
                        let error = inp.0.clone() + " has not been found in the anonymous call";
                        return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),error));
                    } else {
                        let pos = names.iter().position(|r| *r == inp.0).unwrap();
                        new_signals.push(signals.get(pos).unwrap().clone());
                        new_operators.push(*operators.get(pos).unwrap());
                    }
                }
            }
            else{
                new_signals = signals.clone();
                for _i in 0..signals.len() {
                    new_operators.push(AssignOp::AssignConstraintSignal);
                }
            }
            if inputs.len() != new_signals.len() || inputs.len() != signals.len() {
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"The number of template input signals must coincide with the number of input parameters ".to_string()));
            }
            for inp in inputs{
                let mut acc = if var_access.is_none(){
                    Vec::new()
                } else{
                    vec![build_array_access(var_access.as_ref().unwrap().clone())]
                };
                acc.push(Access::ComponentAccess(inp.0.clone()));
                let  (mut stmts, mut declarations2, new_exp) = remove_anonymous_from_expression(
                        &mut templates.clone(), 
                        file_lib, 
                        new_signals.get(i).unwrap().clone(),
                        var_access
                )?;
                if new_exp.contains_anonymous_comp() {
                    return Result::Err(AnonymousCompError::anonymous_general_error(new_exp.get_meta().clone(),"Inputs of an anonymous component cannot contain anonymous calls".to_string()));                
                }
                seq_substs.append(&mut stmts);
                declarations.append(&mut declarations2);
                let subs = Statement::Substitution { meta: meta.clone(), var: id_anon_temp.clone(), access: acc, op: *new_operators.get(i).unwrap(), rhe: new_exp };
                i+=1;
                seq_substs.push(subs);
            }
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
                Result::Ok((vec![Statement::Block { meta: meta, stmts: seq_substs }],declarations,out_exp))

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
                Result::Ok((vec![Statement::Block { meta: meta, stmts: seq_substs }], declarations, out_exp))

            }
        },
        Tuple { meta, values } => {
            let mut new_values = Vec::new();
            let mut new_stmts : Vec<Statement> = Vec::new();
            let mut declarations : Vec<Statement> = Vec::new();
            for val in values{
                let result = remove_anonymous_from_expression(templates, file_lib, val, var_access);
                match result {
                    Ok((mut stm, mut declaration, val2)) => {
                        new_stmts.append(&mut stm);
                        new_values.push(val2);
                        declarations.append(&mut declaration);
                    },
                    Err(er) => {return Result::Err(er);},
                }
            }
            Result::Ok((new_stmts, declarations, build_tuple(meta, new_values)))
        },
        ParallelOp { meta, rhe } => {
            if !rhe.is_call() && !rhe.is_anonymous_comp() && rhe.contains_anonymous_comp() {
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"Bad use of parallel operator in combination with anonymous components ".to_string()));       
            }
            else if rhe.is_call() && rhe.contains_anonymous_comp() {
                return Result::Err(AnonymousCompError::anonymous_general_error(meta.clone(),"An anonymous component cannot be used as a parameter in a template call ".to_string()));
            }
            else if rhe.is_anonymous_comp(){
                let rhe2 = rhe.make_anonymous_parallel();
                return remove_anonymous_from_expression(templates, file_lib, rhe2, var_access);
            }
            Result::Ok((Vec::new(),Vec::new(),exp))
        },
    }
}

pub fn separate_declarations_in_comp_var_subs(declarations: Vec<Statement>) -> (Vec<Statement>, Vec<Statement>, Vec<Statement>){
    let mut components_dec = Vec::new();
    let mut variables_dec = Vec::new();
    let mut substitutions = Vec::new();
    for dec in declarations {
        if let Statement::Declaration {  ref xtype, .. } = dec {
            if VariableType::Component.eq(&xtype) || VariableType::AnonymousComponent.eq(&xtype){
                components_dec.push(dec);
            }
            else if VariableType::Var.eq(&xtype) {
                variables_dec.push(dec);
            }
            else {
                unreachable!();
            }
        }
        else if let Statement::Substitution {.. } = dec {
            substitutions.push(dec);
        } else{
            unreachable!();
        }
    }
    (components_dec, variables_dec, substitutions)
}
pub fn apply_syntactic_sugar(program_archive : &mut  ProgramArchive) -> Result<(),Report> {
    let mut new_templates : HashMap<String, TemplateData> = HashMap::new();
    if program_archive.get_main_expression().is_anonymous_comp() {
        return Result::Err(AnonymousCompError::anonymous_general_error(program_archive.get_main_expression().get_meta().clone(),"The main component cannot contain an anonymous call  ".to_string()));
     
    }
    for temp in program_archive.templates.clone() {
        let t = temp.1.clone();
        let body = t.get_body().clone();
        let (new_body, initializations) = remove_anonymous_from_statement(&mut program_archive.templates, &program_archive.file_library, body, &None)?;
        if let Statement::Block { meta, mut stmts } = new_body {
            let (component_decs, variable_decs, mut substitutions) = separate_declarations_in_comp_var_subs(initializations);
            let mut init_block = vec![
                build_initialization_block(meta.clone(), VariableType::Var, variable_decs),
                build_initialization_block(meta.clone(), VariableType::Component, component_decs)];
            init_block.append(&mut substitutions);
            init_block.append(&mut stmts);
            let new_body_with_inits = build_block(meta, init_block);
            let new_body = remove_tuples_from_statement(&mut program_archive.templates, &program_archive.file_library, new_body_with_inits)?;
            let t2 = TemplateData::copy(t.get_name().to_string(), t.get_file_id(), new_body, t.get_num_of_params(), t.get_name_of_params().clone(),
                                t.get_param_location(), t.get_inputs().clone(), t.get_outputs().clone(), t.is_parallel(), t.is_custom_gate(), t.get_declaration_inputs().clone(), t.get_declaration_outputs().clone());
            new_templates.insert(temp.0.clone(), t2);            
        } else{
            unreachable!()
        }
    }
    program_archive.templates = new_templates;
    Result::Ok(())
}

fn remove_tuples_from_statement(templates: &mut HashMap<String, TemplateData>, file_lib: &FileLibrary, stm: Statement) -> Result<Statement, Report> {
   match stm.clone() {
        Statement::MultSubstitution { meta, lhe, op, rhe  } => {
            let ok = remove_tuple_from_expression(templates, file_lib, lhe)?;
            let ok2 = remove_tuple_from_expression(templates, file_lib, rhe)?;
            match (ok, ok2) {
                (Expression::Tuple { values: mut values1, .. },
                    Expression::Tuple { values: mut values2, .. }) => {
                    if values1.len() == values2.len() {
                        let mut substs = Vec::new();
                        while  values1.len() > 0 {
                            let lhe = values1.remove(0);
                            if let Expression::Variable { meta, name, access } = lhe {  
                                let rhe = values2.remove(0);
                                if name != "_" {                                
                                    substs.push(build_substitution(meta.clone(), name.clone(), access.to_vec(), op, rhe));
                                }
                            } else{   
                                return Result::Err(TupleError::tuple_general_error(meta.clone(),"The elements of the receiving tuple must be signals or variables.".to_string()));
                            }
                        }
                        return Result::Ok(build_block(meta.clone(),substs));
                    } else if values1.len() > 0 {
                        return Result::Err(TupleError::tuple_general_error(meta.clone(),"The number of elements in both tuples does not coincide".to_string()));           
                    } else {
                        return Result::Err(TupleError::tuple_general_error(meta.clone(),"This expression must be in the right side of an assignment".to_string()));           
                    }
                },
                (lhe, rhe) => { 
                    if lhe.is_tuple() || lhe.is_variable(){
                        return Result::Err(TupleError::tuple_general_error(rhe.get_meta().clone(),"This expression must be a tuple or an anonymous component".to_string()));
                    } else {
                        return Result::Err(TupleError::tuple_general_error(lhe.get_meta().clone(),"This expression must be a tuple, a component, a signal or a variable ".to_string()));    
                    }
                }
            }
        },
        Statement::IfThenElse { meta, cond, if_case, else_case } 
        => { 
            if cond.contains_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used inside a condition ".to_string()));       
              } else{
                let if_ok = remove_tuples_from_statement(templates, file_lib, *if_case)?;
                let b_if = Box::new(if_ok);
                if else_case.is_none(){
                    Result::Ok(Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::None})
                }else {
                    let else_c = *(else_case.unwrap());
                    let else_ok = remove_tuples_from_statement(templates, file_lib, else_c)?;
                    let b_else = Box::new(else_ok);
                    Result::Ok(Statement::IfThenElse { meta : meta, cond : cond, if_case: b_if, else_case: Option::Some(b_else)})
                }
            }
        }

        Statement::While { meta, cond, stmt }   => {
            if cond.contains_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used inside a condition ".to_string()));       
           } else{
                let while_ok = remove_tuples_from_statement(templates, file_lib, *stmt)?;
                let b_while = Box::new(while_ok);
                Result::Ok(Statement::While { meta : meta, cond : cond, stmt : b_while})
            }
        }     
        Statement::LogCall {meta, args } => {
            let mut newargs = Vec::new();
            for arg in args {
                match arg {
                    LogArgument::LogStr(str) => {newargs.push(LogArgument::LogStr(str));},
                    LogArgument::LogExp(exp) => {
                            let mut args2 = separate_tuple_for_logcall(vec![exp]);
                            newargs.append(&mut args2);
                    },
                }
            }
            Result::Ok(build_log_call(meta, newargs))
        }  
        Statement::Assert { meta, arg}   => { Result::Ok(build_assert(meta, arg))}
        Statement::Return {  meta, value: arg}=> {
            if arg.contains_tuple(){
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used inside a function ".to_string()));       
            }
            else{ Result::Ok(build_return(meta, arg))}
        }
        Statement::ConstraintEquality {meta, lhe, rhe } => {
            if lhe.contains_tuple() || rhe.contains_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used with the operator === ".to_string()));       
            }
            else{ Result::Ok(build_constraint_equality(meta, lhe, rhe)) }
        }
        Statement::Declaration { meta , xtype , name ,
                                 dimensions, .. } =>
        {
            for exp in dimensions.clone(){
                if exp.contains_tuple(){
                    return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array ".to_string()));       
                 }
            }
            Result::Ok(build_declaration(meta, xtype, name, dimensions))
        }
        Statement::InitializationBlock { meta, xtype, initializations } =>
        {
            let mut new_inits = Vec::new();
            for stmt in initializations {
                let stmt_ok = remove_tuples_from_statement(templates, file_lib, stmt)?;
                new_inits.push(stmt_ok);
            }
            Result::Ok(Statement::InitializationBlock { meta: meta, xtype: xtype, initializations: new_inits })
        }
        Statement::Block { meta, stmts } => { 
            let mut new_stmts = Vec::new();
            for stmt in stmts {
                let stmt_ok = remove_tuples_from_statement(templates, file_lib, stmt)?;
                new_stmts.push(stmt_ok);
            }
            Result::Ok(Statement::Block { meta : meta, stmts: new_stmts})
        }
        Statement::Substitution {  meta, var, op, rhe, access} => {
            let new_rhe = remove_tuple_from_expression(templates, file_lib, rhe)?;
             if new_rhe.is_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"Left-side of the statement is not a tuple".to_string()));       
            }
            if var != "_" {   
                Result::Ok(Statement::Substitution { meta: meta.clone(), var: var, access: access, op: op, rhe: new_rhe })
            }
            else {//If this
                Result::Ok(build_block(meta, Vec::new()))
            }
        }
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


pub fn remove_tuple_from_expression(templates : &mut HashMap<String, TemplateData>, file_lib : & FileLibrary, exp : Expression) -> Result<Expression,Report>{
    use Expression::*;
    match exp.clone() {
        ArrayInLine { meta, values } => {    
        for value in values{
            if value.contains_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array ".to_string()));       
            }
        }
        Result::Ok(exp)
        }, 
        UniformArray { meta, value, dimension } => {
            if value.contains_tuple() || dimension.contains_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used to define a dimension of an array ".to_string()));       
            }
            Result::Ok(exp)
        },
        Number(_, _) => { Result::Ok(exp) },
        Variable { meta, .. } => {
            if exp.contains_tuple(){
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used to access an array ".to_string()));       
   
            }
            Result::Ok(exp)
        },
        InfixOp { meta, lhe, rhe, .. } => {
            if lhe.contains_tuple() || rhe.contains_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used in the middle of an operation".to_string()));       
            }
            Result::Ok(exp)
        },
        PrefixOp { meta, rhe, .. } => {
            if rhe.contains_tuple()  {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used in the middle of an operation".to_string()));       
            }
            Result::Ok(exp)
        },
        InlineSwitchOp { meta, cond, if_true,  if_false } => {
            if cond.contains_tuple() || if_true.contains_tuple() || if_false.contains_tuple() {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used inside an inline switch".to_string()));       
            }
            Result::Ok(exp)
        },
        Call { meta, args, .. } => {
            for value in args{
                if value.contains_tuple() {
                    return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used as a parameter of a function call".to_string()));       
                }
            }
            Result::Ok(exp)
        },
        AnonymousComp { .. } => {
            unreachable!();
        }
        Tuple { meta, values } => {
            let mut unfolded_values =  Vec::new();
            for val in values {
                let exp = remove_tuple_from_expression(templates, file_lib, val)?;
                if let Tuple { values: mut values2, ..} = exp {
                    unfolded_values.append(&mut values2);
                }  else {
                    unfolded_values.push(exp);
                }                               
            }
            Result::Ok(build_tuple(meta, unfolded_values))
        },
        ParallelOp { meta, rhe} => {
            if rhe.contains_tuple()  {
                return Result::Err(TupleError::tuple_general_error(meta.clone(),"A tuple cannot be used in a parallel operator ".to_string()));       
            }
            Result::Ok(exp)
        },
    }
}
