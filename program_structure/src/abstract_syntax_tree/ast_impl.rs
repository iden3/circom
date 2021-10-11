use super::ast::*;

impl AST {
    pub fn get_includes(&self) -> &Vec<String> {
        &self.includes
    }

    pub fn get_version(&self) -> &Option<Version> {
        &self.compiler_version
    }

    pub fn get_definitions(&self) -> &Vec<Definition> {
        &self.definitions
    }
    pub fn decompose(self) -> (Meta, Option<Version>, Vec<String>, Vec<Definition>, Option<MainComponent>) {
        (self.meta, self.compiler_version, self.includes, self.definitions, self.main_component)
    }
}
