use super::ast::{Definition, Expression, MainComponent};
use super::file_definition::{FileID, FileLibrary};
use super::function_data::{FunctionData, FunctionInfo};
use super::program_merger::Merger;
use super::template_data::{TemplateData, TemplateInfo};
use crate::abstract_syntax_tree::ast::FillMeta;
use std::collections::HashSet;
use crate::error_definition::Report;

type Contents = Vec<(FileID, Vec<Definition>)>;

#[derive(Clone)]
pub struct ProgramArchive {
    pub id_max: usize,
    pub file_id_main: FileID,
    pub file_library: FileLibrary,
    pub functions: FunctionInfo,
    pub templates: TemplateInfo,
    pub function_keys: HashSet<String>,
    pub template_keys: HashSet<String>,
    pub public_inputs: Vec<String>,
    pub initial_template_call: Expression,
}
impl ProgramArchive {
    pub fn new(
        file_library: FileLibrary,
        file_id_main: FileID,
        main_component: MainComponent,
        program_contents: Contents,
    ) -> Result<ProgramArchive, (FileLibrary, Vec<Report>)> {
        let mut merger = Merger::new();
        let mut reports = vec![];
        for (file_id, definitions) in program_contents {
            if let Err(mut errs) = merger.add_definitions(file_id, definitions) {
                reports.append(&mut errs);
            }
        }
        let (mut fresh_id, functions, templates) = merger.decompose();
        let mut function_keys = HashSet::new();
        let mut template_keys = HashSet::new();
        for key in functions.keys() {
            function_keys.insert(key.clone());
        }
        for key in templates.keys() {
            template_keys.insert(key.clone());
        }
        let (public_inputs, mut initial_template_call) = main_component;
        initial_template_call.fill(file_id_main, &mut fresh_id);
        if reports.is_empty() {
            Ok(ProgramArchive {
                id_max: fresh_id,
                file_id_main,
                file_library,
                functions,
                templates,
                public_inputs,
                initial_template_call,
                function_keys,
                template_keys,
            })
        } else {
            Err((file_library, reports))
        }

    }
    //file_id_main
    pub fn get_file_id_main(&self) -> &FileID {
        &self.file_id_main
    }
    //template functions
    pub fn contains_template(&self, template_name: &str) -> bool {
        self.templates.contains_key(template_name)
    }
    pub fn get_template_data(&self, template_name: &str) -> &TemplateData {
        assert!(self.contains_template(template_name));
        self.templates.get(template_name).unwrap()
    }
    pub fn get_mut_template_data(&mut self, template_name: &str) -> &mut TemplateData {
        assert!(self.contains_template(template_name));
        self.templates.get_mut(template_name).unwrap()
    }
    pub fn get_template_names(&self) -> &HashSet<String> {
        &self.template_keys
    }
    pub fn get_templates(&self) -> &TemplateInfo {
        &self.templates
    }
    pub fn get_mut_templates(&mut self) -> &mut TemplateInfo {
        &mut self.templates
    }

    pub fn remove_template(&mut self, id: &str) {
        self.template_keys.remove(id);
        self.templates.remove(id);
    }

    //functions functions
    pub fn contains_function(&self, function_name: &str) -> bool {
        self.get_functions().contains_key(function_name)
    }
    pub fn get_function_data(&self, function_name: &str) -> &FunctionData {
        assert!(self.contains_function(function_name));
        self.get_functions().get(function_name).unwrap()
    }
    pub fn get_mut_function_data(&mut self, function_name: &str) -> &mut FunctionData {
        assert!(self.contains_function(function_name));
        self.functions.get_mut(function_name).unwrap()
    }
    pub fn get_function_names(&self) -> &HashSet<String> {
        &self.function_keys
    }
    pub fn get_functions(&self) -> &FunctionInfo {
        &self.functions
    }
    pub fn get_mut_functions(&mut self) -> &mut FunctionInfo {
        &mut self.functions
    }
    pub fn remove_function(&mut self, id: &str) {
        self.function_keys.remove(id);
        self.functions.remove(id);
    }

    //main_component functions
    pub fn get_public_inputs_main_component(&self) -> &Vec<String> {
        &self.public_inputs
    }
    pub fn get_main_expression(&self) -> &Expression {
        &self.initial_template_call
    }
    // FileLibrary functions
    pub fn get_file_library(&self) -> &FileLibrary {
        &self.file_library
    }
}
