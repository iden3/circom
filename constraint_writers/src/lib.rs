pub mod debug_writer;
pub mod json_writer;
pub mod log_writer;
pub mod r1cs_writer;
pub mod sym_writer;

pub trait ConstraintExporter {
    #[allow(clippy::result_unit_err)]
    fn r1cs(&self, out: &str, custom_gates: bool) -> Result<(), ()>;
    #[allow(clippy::result_unit_err)]
    fn json_constraints(&self, writer: &debug_writer::DebugWriter) -> Result<(), ()>;
    #[allow(clippy::result_unit_err)]
    fn sym(&self, out: &str) -> Result<(), ()>;
}
