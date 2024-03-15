use super::json_writer::ConstraintJSON;

#[derive(Clone)]
pub struct DebugWriter {
    pub json_constraints: String,
}
impl DebugWriter {
    #[allow(clippy::result_unit_err)]
    pub fn new(c: String) -> Result<DebugWriter, ()> {
        Result::Ok(DebugWriter { json_constraints: c })
    }

    #[allow(clippy::result_unit_err)]
    pub fn build_constraints_file(&self) -> Result<ConstraintJSON, ()> {
        ConstraintJSON::new(&self.json_constraints)
    }
}
