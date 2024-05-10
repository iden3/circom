use super::json_writer::ConstraintJSON;

#[derive(Clone)]
pub struct DebugWriter {
    pub json_constraints: String,
}
impl DebugWriter {
    pub fn new(c: String) -> Result<DebugWriter, ()> {
        Result::Ok(DebugWriter { json_constraints: c })
    }

    pub fn build_constraints_file(&self, fs: &dyn vfs::FileSystem) -> Result<ConstraintJSON, ()> {
        ConstraintJSON::new(fs, &self.json_constraints)
    }
}
