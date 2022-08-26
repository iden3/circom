use super::input_user::Input;
use program_structure::error_definition::Report;
use program_structure::program_archive::ProgramArchive;
use crate::VERSION;


pub fn parse_project(input_info: &Input) -> Result<ProgramArchive, ()> {
    let initial_file = input_info.input_file().to_string();
    let result_program_archive = parser::run_parser(initial_file, VERSION, input_info.get_link_libraries().to_vec());
    match result_program_archive {
        Result::Err((file_library, report_collection)) => {
            Report::print_reports(&report_collection, &file_library);
            Result::Err(())
        }
        Result::Ok((program_archive, warnings)) => {
            Report::print_reports(&warnings, &program_archive.file_library);
            Result::Ok(program_archive)
        }
    }
}
