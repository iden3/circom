use super::ast;
use super::ast::{FillMeta, Statement};
use super::file_definition::{FileID, FileLocation};
use super::wire_data::*;
use std::collections::{HashMap};

pub type TemplateInfo = HashMap<String, TemplateData>;

#[derive(Clone)]
pub struct TemplateData {
    file_id: FileID,
    name: String,
    body: Statement,
    num_of_params: usize,
    name_of_params: Vec<String>,
    param_location: FileLocation,
    input_wires: WireInfo,
    output_wires: WireInfo,
    is_parallel: bool,
    is_custom_gate: bool,
    /* Only used to know the order in which signals are declared.*/
    input_declarations: WireDeclarationOrder,
    output_declarations: WireDeclarationOrder,
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
        is_custom_gate: bool,
    ) -> TemplateData {
        body.fill(file_id, elem_id);
        let mut input_wires = WireInfo::new();
        let mut output_wires = WireInfo::new();
        let mut input_declarations = WireDeclarationOrder::new();
        let mut output_declarations = WireDeclarationOrder::new();
        fill_inputs_and_outputs(&body, &mut input_wires, &mut output_wires, &mut input_declarations, &mut output_declarations);
        TemplateData {
            name,
            file_id,
            body,
            num_of_params,
            name_of_params,
            param_location,
            input_wires,
            output_wires,
            is_parallel,
            is_custom_gate,
            input_declarations,
            output_declarations
        }
    }

    pub fn copy(
        name: String,
        file_id: FileID,
        body: Statement,
        num_of_params: usize,
        name_of_params: Vec<String>,
        param_location: FileLocation,
        input_wires: WireInfo,
        output_wires: WireInfo,
        is_parallel: bool,
        is_custom_gate: bool,
        input_declarations: WireDeclarationOrder,
        output_declarations: WireDeclarationOrder
    ) -> TemplateData {
        TemplateData {
            name,
            file_id,
            body,
            num_of_params,
            name_of_params,
            param_location,
            input_wires,
            output_wires,
            is_parallel,
            is_custom_gate,
            input_declarations,
            output_declarations
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
    pub fn set_body(&mut self, body: Statement){
        self.body = body;
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
    pub fn get_input_info(&self, name: &str) -> Option<&WireData> {
        self.input_wires.get(name)
    }
    pub fn get_output_info(&self, name: &str) -> Option<&WireData> {
        self.output_wires.get(name)
    }
    pub fn get_inputs(&self) -> &WireInfo {
        &self.input_wires
    }
    pub fn get_outputs(&self) -> &WireInfo {
        &self.output_wires
    }
    pub fn get_declaration_inputs(&self) -> &WireDeclarationOrder {
        &self.input_declarations
    }
    pub fn get_declaration_outputs(&self) -> &WireDeclarationOrder {
        &self.output_declarations
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn is_parallel(&self) -> bool {
        self.is_parallel
    }
    pub fn is_custom_gate(&self) -> bool {
        self.is_custom_gate
    }
}

fn fill_inputs_and_outputs(
    template_statement: &Statement,
    input_wires: &mut WireInfo,
    output_wires: &mut WireInfo,
    input_declarations: &mut WireDeclarationOrder,
    output_declarations: &mut WireDeclarationOrder
) {
    use Statement::*;
    match template_statement {
        IfThenElse { if_case, else_case, .. } => {
            fill_inputs_and_outputs(if_case, input_wires, output_wires, input_declarations, output_declarations);
            if let Option::Some(else_value) = else_case {
                fill_inputs_and_outputs(else_value, input_wires, output_wires, input_declarations, output_declarations);
            }
        }
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                fill_inputs_and_outputs(stmt, input_wires, output_wires, input_declarations, output_declarations);
            }
        }
        While { stmt, .. } => {
            fill_inputs_and_outputs(stmt, input_wires, output_wires, input_declarations, output_declarations);
        }
        InitializationBlock { initializations, .. } => {
            for initialization in initializations.iter() {
                fill_inputs_and_outputs(initialization, input_wires, output_wires, input_declarations, output_declarations);
            }
        }
        Declaration { xtype, name, dimensions, .. } => {
            match xtype {
                ast::VariableType::Signal(stype, tag_list) => {
                    let wire_name = name.clone();
                    let dim = dimensions.len();
                    let mut tag_info = TagInfo::new();
                    for tag in tag_list{
                        tag_info.insert(tag.clone());
                    }
                    let wire_data = WireData::new(WireType::Signal,dim,tag_info);

                    match stype {
                        ast::SignalType::Input => {
                            input_wires.insert(wire_name.clone(), wire_data);
                            input_declarations.push((wire_name,dim));
                        }
                        ast::SignalType::Output => {
                            output_wires.insert(wire_name.clone(), wire_data);
                            output_declarations.push((wire_name,dim));
                        }
                        _ => {} //no need to deal with intermediate signals
                    }
                },
                ast::VariableType::Bus(tname, stype, tag_list) => {
                    let wire_name = name.clone();
                    let dim = dimensions.len();
                    let type_name = tname.clone();
                    let mut tag_info = TagInfo::new();
                    for tag in tag_list{
                        tag_info.insert(tag.clone());
                    }
                    let wire_data = WireData::new(WireType::Bus(type_name),dim,tag_info);

                    match stype {
                        ast::SignalType::Input => {
                            input_wires.insert(wire_name.clone(), wire_data);
                            input_declarations.push((wire_name,dim));
                        }
                        ast::SignalType::Output => {
                            output_wires.insert(wire_name.clone(), wire_data);
                            output_declarations.push((wire_name,dim));
                        }
                        _ => {} //no need to deal with intermediate signals
                    }
                },
                _ => {},
            }
        }
        _ => {}
    }
}
