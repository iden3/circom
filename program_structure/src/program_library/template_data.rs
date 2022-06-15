use super::ast;
use super::ast::{FillMeta, SignalElementType, Statement};
use super::file_definition::FileID;
use crate::file_definition::FileLocation;
use std::collections::hash_map::HashMap;

pub type TemplateInfo = HashMap<String, TemplateData>;
type SignalInfo = HashMap<String, (usize, SignalElementType)>;

#[derive(Clone)]
pub struct TemplateData {
    file_id: FileID,
    name: String,
    body: Statement,
    num_of_params: usize,
    name_of_params: Vec<String>,
    param_location: FileLocation,
    input_signals: SignalInfo,
    output_signals: SignalInfo,
    is_parallel: bool,
}

impl TemplateData {
    pub fn new(
        name: String,
        file_id: FileID,
        mut body: Statement,
        num_of_params: usize,
        name_of_params: Vec<String>,
        param_location: FileLocation,
        elem_id: &mut usize,
        is_parallel: bool,
    ) -> TemplateData {
        body.fill(file_id, elem_id);
        let mut input_signals = SignalInfo::new();
        let mut output_signals = SignalInfo::new();
        fill_inputs_and_outputs(&body, &mut input_signals, &mut output_signals);
        TemplateData {
            name,
            file_id,
            body,
            num_of_params,
            name_of_params,
            param_location,
            input_signals,
            output_signals,
            is_parallel,
        }
    }
    pub fn get_file_id(&self) -> FileID {
        self.file_id
    }
    pub fn get_body(&self) -> &Statement {
        &self.body
    }
    pub fn get_body_as_vec(&self) -> &Vec<Statement> {
        match &self.body {
            Statement::Block { stmts, .. } => stmts,
            _ => panic!("Function body should be a block"),
        }
    }
    pub fn get_mut_body(&mut self) -> &mut Statement {
        &mut self.body
    }
    pub fn get_mut_body_as_vec(&mut self) -> &mut Vec<Statement> {
        match &mut self.body {
            Statement::Block { stmts, .. } => stmts,
            _ => panic!("Function body should be a block"),
        }
    }
    pub fn get_num_of_params(&self) -> usize {
        self.num_of_params
    }
    pub fn get_param_location(&self) -> FileLocation {
        self.param_location.clone()
    }
    pub fn get_name_of_params(&self) -> &Vec<String> {
        &self.name_of_params
    }
    pub fn get_input_info(&self, name: &str) -> Option<&(usize, SignalElementType)> {
        self.input_signals.get(name)
    }
    pub fn get_output_info(&self, name: &str) -> Option<&(usize, SignalElementType)> {
        self.output_signals.get(name)
    }
    pub fn get_inputs(&self) -> &SignalInfo {
        &self.input_signals
    }
    pub fn get_outputs(&self) -> &SignalInfo {
        &self.output_signals
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn is_parallel(&self) -> bool {
        self.is_parallel
    }
}

fn fill_inputs_and_outputs(
    template_statement: &Statement,
    input_signals: &mut SignalInfo,
    output_signals: &mut SignalInfo,
) {
    match template_statement {
        Statement::IfThenElse { if_case, else_case, .. } => {
            fill_inputs_and_outputs(if_case, input_signals, output_signals);
            if let Option::Some(else_value) = else_case {
                fill_inputs_and_outputs(else_value, input_signals, output_signals);
            }
        }
        Statement::Block { stmts, .. } => {
            for stmt in stmts.iter() {
                fill_inputs_and_outputs(stmt, input_signals, output_signals);
            }
        }
        Statement::While { stmt, .. } => {
            fill_inputs_and_outputs(stmt, input_signals, output_signals);
        }
        Statement::InitializationBlock { initializations, .. } => {
            for initialization in initializations.iter() {
                fill_inputs_and_outputs(initialization, input_signals, output_signals);
            }
        }
        Statement::Declaration { xtype, name, dimensions, .. } => {
            if let ast::VariableType::Signal(stype, tag) = xtype {
                let signal_name = name.clone();
                let dim = dimensions.len();
                match stype {
                    ast::SignalType::Input => {
                        input_signals.insert(signal_name, (dim, *tag));
                    }
                    ast::SignalType::Output => {
                        output_signals.insert(signal_name, (dim, *tag));
                    }
                    _ => {} //no need to deal with intermediate signals
                }
            }
        }
        _ => {}
    }
}
