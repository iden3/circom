pub mod debug_writer;
pub mod json_writer;
pub mod log_writer;
pub mod r1cs_writer;
pub mod sym_writer;

pub trait ConstraintExporter {
    fn r1cs(&self, fs: &dyn vfs::FileSystem, out: &str, custom_gates: bool) -> Result<(), ()>;
    fn json_constraints(&self, fs: &dyn vfs::FileSystem, writer: &debug_writer::DebugWriter) -> Result<(), ()>;
    fn sym(&self, fs: &dyn vfs::FileSystem, out: &str) -> Result<(), ()>;
}
