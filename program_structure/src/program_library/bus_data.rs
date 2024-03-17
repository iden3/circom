use super::ast::{FillMeta, Statement, VariableType, SignalType};
use super::file_definition::{FileID, FileLocation};
use super::wire_data::*;
use std::collections::{HashMap};

pub type BusInfo = HashMap<String, BusData>;

#[derive(Clone)]
pub struct BusData {
    name: String,
    file_id: FileID,
    num_of_params: usize,
    name_of_params: Vec<String>,
    param_location: FileLocation,
    fields: WireInfo,
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
        let fields = WireInfo::new();

        BusData {
             name, 
             file_id, 
             body, 
             name_of_params, 
             param_location, 
             num_of_params,
             fields
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
    pub fn get_field_info(&self, name: &str) -> Option<&WireData> {
        self.fields.get(name)
    }
    pub fn get_fields(&self) -> &WireInfo {
        &self.fields
    }
}


fn fill_fields(
    bus_statement: &Statement,
    fields: &mut WireInfo,
) {
    use Statement::*;
    match bus_statement {
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                fill_fields(stmt, fields);
            }
        }
        Declaration { xtype, name, dimensions, .. } => {
            match xtype {
                VariableType::Signal(stype, tag_list) | VariableType::Bus(stype, tag_list) => {
                    if *stype == SignalType::Intermediate {
                        let wire_name = name.clone();
                        let dim = dimensions.len();
                        let mut tag_info = TagInfo::new();
                        for tag in tag_list {
                            tag_info.insert(tag.clone());
                        }
                        let field_data = if let VariableType::Signal(_,_) = xtype {
                            WireData::new(WireType::Signal,dim,tag_info)
                        } else {
                            WireData::new(WireType::Bus,dim,tag_info)
                        };
                        fields.insert(wire_name, field_data);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}