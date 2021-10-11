use super::ast::Definition;
use super::error_code::ReportCode;
use super::error_definition::Report;
use super::file_definition::FileID;
use super::function_data::{FunctionData, FunctionInfo};
use super::template_data::{TemplateData, TemplateInfo};

pub struct Merger {
    fresh_id: usize,
    function_info: FunctionInfo,
    template_info: TemplateInfo,
}
impl Default for Merger {
    fn default() -> Self {
        Merger {
            fresh_id: 0,
            function_info: FunctionInfo::new(),
            template_info: TemplateInfo::new(),
        }
    }
}

impl Merger {
    pub fn new() -> Merger {
        Merger::default()
    }

    pub fn add_definitions(&mut self, file_id: FileID, definitions: Vec<Definition>)  -> Result<(), Vec<Report>> {
        let mut reports = vec![];
        for definition in definitions {
            let (name, meta) = match definition {
                Definition::Template { name, args, arg_location, body, meta, parallel } => {
                    if self.contains_function(&name) || self.contains_template(&name) {
                        (Option::Some(name), meta)
                    } else {
                        let new_data = TemplateData::new(
                            name.clone(),
                            file_id,
                            body,
                            args.len(),
                            args,
                            arg_location,
                            &mut self.fresh_id,
                            parallel,
                        );
                        self.get_mut_template_info().insert(name.clone(), new_data);
                        (Option::None, meta)
                    }
                }
                Definition::Function { name, body, args, arg_location, meta } => {
                    if self.contains_function(&name) || self.contains_template(&name) {
                        (Option::Some(name), meta)
                    } else {
                        let new_data = FunctionData::new(
                            name.clone(),
                            file_id,
                            body,
                            args.len(),
                            args,
                            arg_location,
                            &mut self.fresh_id,
                        );
                        self.get_mut_function_info().insert(name.clone(), new_data);
                        (Option::None, meta)
                    }
                }
            };
            if let Option::Some(definition_name) = name {
                let mut report = Report::error(
                    String::from("Duplicated callable symbol"),
                    ReportCode::SameSymbolDeclaredTwice,
                );
                report.add_primary(
                    meta.file_location(),
                    file_id,
                    format!("{} is already in use", definition_name),
                );
                reports.push(report);
            }
        }
        if reports.is_empty() { Ok(()) } else { Err(reports) }
    }
    pub fn contains_function(&self, function_name: &str) -> bool {
        self.get_function_info().contains_key(function_name)
    }
    fn get_function_info(&self) -> &FunctionInfo {
        &self.function_info
    }
    fn get_mut_function_info(&mut self) -> &mut FunctionInfo {
        &mut self.function_info
    }

    pub fn contains_template(&self, template_name: &str) -> bool {
        self.get_template_info().contains_key(template_name)
    }
    fn get_template_info(&self) -> &TemplateInfo {
        &self.template_info
    }
    fn get_mut_template_info(&mut self) -> &mut TemplateInfo {
        &mut self.template_info
    }


    pub fn decompose(self) -> (usize, FunctionInfo, TemplateInfo) {
        (self.fresh_id, self.function_info, self.template_info)
    }
}
