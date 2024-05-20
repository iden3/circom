use super::json_writer::ConstraintJSON;

#[derive(Clone)]
pub struct DebugWriter {}

impl DebugWriter {
    pub fn new() -> DebugWriter {
        DebugWriter {}
    }

    pub fn build_constraints_file(&self) -> ConstraintJSON {
        ConstraintJSON::new()
    }
}
