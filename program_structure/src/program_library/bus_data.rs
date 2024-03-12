use super::ast;
use super::ast::{FillMeta, Statement};
use super::file_definition::FileID;
use crate::file_definition::FileLocation;
use std::collections::{HashMap, BTreeMap, HashSet};

pub type BusInfo = HashMap<String, BusData>;
pub type TagInfo = HashSet<String>;
type SignalInfo = BTreeMap<String, (usize, TagInfo)>;
type SignalDeclarationOrder = Vec<(String, usize)>;
#[derive(Clone)]
pub struct BusData {
    name: String,
    file_id: FileID,
    num_of_params: usize,
    name_of_params: Vec<String>,
    param_location: FileLocation,
    signals: SignalInfo,
    body: Statement,
}

impl BusData {
    pub fn new(
        name: String,
        file_id: FileID,
        mut body: Statement,
        num_of_params: usize,
        name_of_params: Vec<String>,
        param_location: FileLocation,
        elem_id: &mut usize,
    ) -> BusData {
        body.fill(file_id, elem_id);
        let signals = SignalInfo::new();

        BusData {
             name, 
             file_id, 
             body, 
             name_of_params, 
             param_location, 
             num_of_params,
             signals
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
    pub fn set_body(&mut self, body: Statement){
        self.body = body;
    }
    pub fn replace_body(&mut self, new: Statement) -> Statement {
        std::mem::replace(&mut self.body, new)
    }
    pub fn get_mut_body_as_vec(&mut self) -> &mut Vec<Statement> {
        match &mut self.body {
            Statement::Block { stmts, .. } => stmts,
            _ => panic!("Function body should be a block"),
        }
    }
    pub fn get_param_location(&self) -> FileLocation {
        self.param_location.clone()
    }
    pub fn get_num_of_params(&self) -> usize {
        self.num_of_params
    }
    pub fn get_name_of_params(&self) -> &Vec<String> {
        &self.name_of_params
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_signal_info(&self, name: &str) -> Option<&(usize, TagInfo)> {
        self.signals.get(name)
    }
    pub fn get_signals(&self) -> &SignalInfo {
        &self.signals
    }
}


fn fill_signals(
    template_statement: &Statement,
    signals: &mut SignalInfo,
) {
    match template_statement {
        Statement::IfThenElse { if_case, else_case, .. } => {
            fill_signals(if_case, signals);
            if let Option::Some(else_value) = else_case {
                fill_signals(else_value, signals);
            }
        }
        Statement::Block { stmts, .. } => {
            for stmt in stmts.iter() {
                fill_signals(stmt, signals);
            }
        }
        Statement::While { stmt, .. } => {
            fill_signals(stmt, signals);
        }
        Statement::InitializationBlock { initializations, .. } => {
            for initialization in initializations.iter() {
                fill_signals(initialization, signals);
            }
        }
        Statement::Declaration { xtype, name, dimensions, .. } => {
            if let ast::VariableType::Signal(stype, tag_list) = xtype {
                let signal_name = name.clone();
                let dim = dimensions.len();
                let mut tag_info = HashSet::new();
                for tag in tag_list{
                    tag_info.insert(tag.clone());
                }

                signals.insert(signal_name.clone(), (dim, tag_info));
            }
        }
        _ => {}
    }
}