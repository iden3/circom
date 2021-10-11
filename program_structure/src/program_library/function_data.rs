use super::ast::{FillMeta, Statement};
use super::file_definition::FileID;
use crate::file_definition::FileLocation;
use std::collections::HashMap;

pub type FunctionInfo = HashMap<String, FunctionData>;

#[derive(Clone)]
pub struct FunctionData {
    name: String,
    file_id: FileID,
    num_of_params: usize,
    name_of_params: Vec<String>,
    param_location: FileLocation,
    body: Statement,
}

impl FunctionData {
    pub fn new(
        name: String,
        file_id: FileID,
        mut body: Statement,
        num_of_params: usize,
        name_of_params: Vec<String>,
        param_location: FileLocation,
        elem_id: &mut usize,
    ) -> FunctionData {
        body.fill(file_id, elem_id);
        FunctionData { name, file_id, body, name_of_params, param_location, num_of_params }
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
}
