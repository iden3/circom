use program_structure::ast::Meta;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::Report;
use program_structure::file_definition::{FileID, FileLocation};
use program_structure::abstract_syntax_tree::ast::Version;

pub struct UnclosedCommentError {
    pub location: FileLocation,
    pub file_id: FileID,
}

impl UnclosedCommentError {
    pub fn produce_report(error: Self) -> Report {
        let mut report = Report::error(format!("unterminated /* */"), ReportCode::ParseFail);
        report.add_primary(error.location, error.file_id, format!("Comment starts here"));
        report
    }
}

pub struct ParsingError {
    pub location: FileLocation,
    pub file_id: FileID,
    pub msg: String,
}

impl ParsingError {
    pub fn produce_report(error: Self) -> Report {
        let mut report = Report::error(error.msg, ReportCode::ParseFail);
        report.add_primary(error.location, error.file_id, format!("Invalid syntax"));
        report
    }
}

pub struct FileOsError {
    pub path: String,
}
impl FileOsError {
    pub fn produce_report(error: Self) -> Report {
        Report::error(format!("Could not open file {}", error.path), ReportCode::ParseFail)
    }
}

pub struct NoMainError;
impl NoMainError {
    pub fn produce_report() -> Report {
        Report::error(
            format!("No main specified in the project structure"),
            ReportCode::NoMainFoundInProject,
        )
    }
}

pub struct MultipleMainError;
impl MultipleMainError {
    pub fn produce_report() -> Report {
        Report::error(
            format!("Multiple main components in the project structure"),
            ReportCode::MultipleMainInComponent,
        )
    }
}

pub struct CompilerVersionError{
    pub path: String,
    pub required_version: Version,
    pub version: Version,
}
impl CompilerVersionError {
    pub fn produce_report(error: Self) -> Report {
        Report::error(
            format!("File {} requires pragma version {:?} that is not supported by the compiler (version {:?})", error.path, error.required_version, error.version ),
            ReportCode::CompilerVersionError,
        )
    }
}



pub struct NoCompilerVersionWarning{
    pub path: String,
    pub version: Version,
}
impl NoCompilerVersionWarning {
    pub fn produce_report(error: Self) -> Report {
        Report::warning(
            format!("File {} does not include pragma version. Assuming pragma version {:?}", error.path, error.version),
            ReportCode::NoCompilerVersionWarning,
        )
    }
}

pub struct AnonymousCompError{
    pub location: FileLocation,
    pub msg : String
}

impl AnonymousCompError {
    pub fn produce_report( error : Self) -> Report {
        Report::error(
            format!("{}", error.msg),
            ReportCode::AnonymousCompError,
        )
    }

    pub fn anonymous_inside_condition_error(meta : Meta) -> Report {
        let error = AnonymousCompError {msg: "An anonymous component cannot be used inside a condition ".to_string(), location : meta.location.clone()};
                    let mut report = AnonymousCompError::produce_report(error);
                    let file_id = meta.get_file_id().clone();
                    report.add_primary(
                        meta.location,
                        file_id,
                        "This is an anonymous component used inside a condition".to_string(),
                    );
                    report
    }
    
    pub fn anonymous_general_error(meta : Meta, msg : String) -> Report {
        let error = AnonymousCompError {msg, location : meta.location.clone()};
                    let mut report = AnonymousCompError::produce_report(error);
                    let file_id = meta.get_file_id().clone();
                    report.add_primary(
                        meta.location,
                        file_id,
                        "This is the anonymous component whose use is not allowed".to_string(),
                    );
                    report
    }
}

pub struct TupleError{
    pub location: FileLocation,
    pub msg : String
}

impl TupleError {
    pub fn produce_report( error : Self) -> Report {
        Report::error(
            format!("{}", error.msg),
            ReportCode::TupleError,
        )
    }

    pub fn tuple_general_error(meta : Meta, msg : String) -> Report {
        let error = TupleError {msg, location : meta.location.clone()};
                    let mut report = TupleError::produce_report(error);
                    let file_id = meta.get_file_id().clone();
                    report.add_primary(
                        meta.location,
                        file_id,
                        "This is the tuple whose use is not allowed".to_string(),
                    );
                    report
    }
}