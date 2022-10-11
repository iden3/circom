use program_structure::ast::*;
use program_structure::program_archive::ProgramArchive;
use program_structure::template_data::TemplateInfo;
use std::collections::{HashMap, HashSet};

type Environment = HashMap<String, String>;
struct PathInformation {
    components: HashSet<String>,
    environment: Environment,
}

pub fn inference(program_archive: &mut ProgramArchive) {
    let mut template_to_inference = HashMap::new();
    for (name, data) in &program_archive.templates {
        let mut analysis =
            PathInformation { components: HashSet::new(), environment: HashMap::new() };
        infer_component_types(data.get_body(), &program_archive.templates, &mut analysis);
        template_to_inference.insert(name.clone(), analysis.environment);
    }

    for (name, mut inference) in template_to_inference {
        let body = program_archive.get_mut_template_data(&name).get_mut_body();
        apply_inference(body, &mut inference);
    }
}

fn infer_component_types(stmt: &Statement, templates: &TemplateInfo, data: &mut PathInformation) {
    use Statement::*;
    match stmt {
        IfThenElse { if_case, else_case, .. } => {
            infer_component_types(if_case, templates, data);
            if let Some(else_stmt) = else_case {
                infer_component_types(else_stmt, templates, data);
            }
        }
        While { stmt, .. } => {
            infer_component_types(stmt, templates, data);
        }
        Block { stmts, .. } => {
            for s in stmts {
                infer_component_types(s, templates, data);
            }
        }
        InitializationBlock { initializations, .. } => {
            for s in initializations {
                infer_component_types(s, templates, data);
            }
        }
        Declaration { xtype, name, .. }
            if VariableType::Component == *xtype || VariableType::AnonymousComponent == *xtype =>{
            data.components.insert(name.clone());
        }
        Substitution { var, rhe, .. } if data.components.contains(var) => {
            if let Some(template) = into_template_inference(rhe, templates) {
                data.environment.insert(var.clone(), template);
            }
        }
        _ => {}
    }
}

fn into_template_inference(expr: &Expression, templates: &TemplateInfo) -> Option<String> {
    use Expression::*;
    match expr {
        InlineSwitchOp { if_true, if_false, .. } => {
            let mut ret = into_template_inference(if_true, templates);
            if ret.is_none() {
                ret = into_template_inference(if_false, templates);
            }
            ret
        }
        Call { id, .. } if templates.contains_key(id) => Some(id.clone()),
        ParallelOp {rhe, ..} =>{
            into_template_inference(rhe, templates)
        },
        _ => None,
    }
}

fn apply_inference(stmt: &mut Statement, env: &mut Environment) {
    use Statement::*;
    match stmt {
        IfThenElse { if_case, else_case, .. } => {
            apply_inference(if_case, env);
            if let Some(else_stmt) = else_case {
                apply_inference(else_stmt, env);
            }
        }
        While { stmt, .. } => {
            apply_inference(stmt, env);
        }
        Block { stmts, .. } => {
            for s in stmts {
                apply_inference(s, env);
            }
        }
        InitializationBlock { initializations, .. } => {
            for s in initializations {
                apply_inference(s, env);
            }
        }
        Declaration { xtype, name, meta, .. } 
            if VariableType::Component == *xtype || VariableType::AnonymousComponent == *xtype => {
                meta.component_inference = env.remove(name);
        }
        _ => {}
    }
}
