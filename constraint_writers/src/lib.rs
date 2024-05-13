use virtual_fs::{FileSystem, FsResult, VPath};

pub mod debug_writer;
pub mod json_writer;
pub mod log_writer;
pub mod r1cs_writer;
pub mod sym_writer;

pub trait ConstraintExporter {
    fn r1cs(&self, fs: &mut dyn FileSystem, out: &str, custom_gates: bool) -> FsResult<()>;
    fn json_constraints(&self, fs: &mut dyn FileSystem, json_constraints_path: &VPath) -> FsResult<()>;
    fn sym(&self, fs: &mut dyn FileSystem, out: &str) -> FsResult<()>;
}
