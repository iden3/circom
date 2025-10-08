use std::{
    fs::{self, File},
    io::Write,
    os::raw::c_void,
    path::Path,
};
use ansi_term::Color;
use anyhow::Result;
use program_structure::{
    file_definition::{FileID, FileLibrary, FileLocation},
    program_archive::ProgramArchive,
};
use melior::{
    self,
    ir::{operation::OperationLike as _, Location, Module, ValueLike},
};
use llzk::prelude::LlzkContext;

/// Stores necessary context for generating LLZK IR along with the generated `Module`.
/// 'ast: lifetime of the circom AST element
/// 'llzk: lifetime of the `LlzkContext` and generated `Module`
pub struct LlzkCodegen<'ast, 'llzk> {
    files: &'ast FileLibrary,
    context: &'llzk LlzkContext,
    module: Module<'llzk>,
}

/// Helper for generating LLZK IR from a circom `ProgramArchive`.
impl<'ast, 'llzk> LlzkCodegen<'ast, 'llzk> {
    /// Creates a new LLZK code generator to generate code for the given `ProgramArchive`.
    pub fn new(context: &'llzk LlzkContext, program_archive: &'ast ProgramArchive) -> Self {
        let files = &program_archive.file_library;
        let filename = files.get_filename_or_default(program_archive.get_file_id_main());
        let main_file_location = Location::new(&context, &filename, 0, 0);
        let module = llzk::dialect::module::llzk_module(main_file_location);
        Self { files, context, module }
    }

    /// Convert circom location information to MLIR location.
    pub fn get_location(&self, file_id: FileID, file_location: FileLocation) -> Location<'llzk> {
        let filename = self.files.get_filename_or_default(&file_id);
        let line = self.files.get_line(file_location.start, file_id).unwrap_or(0);
        let column = self.files.get_column(file_location.start, file_id).unwrap_or(0);
        Location::new(&self.context, &filename, line, column)
    }

    /// Verify the generated `Module`.
    pub fn verify(&self) -> bool {
        self.module.as_operation().verify()
    }

    /// Write the generated `Module` to a file.
    pub fn write_to_file(&self, filename: &str) -> Result<(), ()> {
        let out_path = Path::new(filename);
        // Ensure parent directories exist
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|_err| {})?;
        }
        let mut file = File::create(out_path).map_err(|_err| {})?;

        unsafe extern "C" fn callback(string_ref: mlir_sys::MlirStringRef, user_data: *mut c_void) {
            let file = &mut *(user_data as *mut File);
            let slice = std::slice::from_raw_parts(string_ref.data as *const u8, string_ref.length);
            let _ = file.write_all(slice).unwrap();
        }

        unsafe {
            // TODO: may need to switch to bytecode at some point. Or add an option for it.
            // mlir_sys::mlirOperationWriteBytecode(
            mlir_sys::mlirOperationPrint(
                self.module.as_operation().to_raw(),
                Some(callback),
                &mut file as *mut File as *mut c_void,
            );
        }
        println!("{} {}", Color::Green.paint("Written successfully:"), filename);
        Result::Ok(())
    }
}

/// A trait to produce LLZK IR from the `ProgramArchive` nodes.
pub trait ProduceLLZK {
    /// Produces LLZK IR from the circom `ProgramArchive` AST element.
    /// 'ret: lifetime of the returned `ValueLike` object
    /// 'ast: lifetime of the circom AST element
    /// 'llzk: lifetime of the `LlzkContext` and generated `Module`
    fn produce_llzk_ir<'ret, 'ast: 'ret, 'llzk: 'ret>(
        &'ast self,
        codegen: &LlzkCodegen<'ast, 'llzk>,
    ) -> Result<Box<dyn ValueLike<'llzk> + 'ret>>;
}

impl ProduceLLZK for ProgramArchive {
    fn produce_llzk_ir<'ret, 'ast: 'ret, 'llzk: 'ret>(
        &'ast self,
        codegen: &LlzkCodegen<'ast, 'llzk>,
    ) -> Result<Box<dyn ValueLike<'llzk> + 'ret>> {
        todo!("Not yet implemented")
    }
}

/// Generate LLZK IR from the given `ProgramArchive` and write it to a file with the given filename.
pub fn generate_llzk(program_archive: &ProgramArchive, filename: &str) -> Result<(), ()> {
    let ctx = LlzkContext::new();
    let codegen = LlzkCodegen::new(&ctx, program_archive);

    // TODO: uncomment when implemented
    // program_archive.produce_llzk_ir(&codegen).expect("Failed to generate LLZK IR");

    // Verify the module and write it to file
    assert!(codegen.verify());
    codegen.write_to_file(filename).expect("Failed to write LLZK code");

    return Result::Ok(());
}
