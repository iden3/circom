use program_structure::ast::Meta;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::Report;
use program_structure::file_definition::{FileID, FileLocation};
use program_structure::abstract_syntax_tree::ast::Version;

pub enum Error {
    UnclosedComment {
        location: FileLocation,
        file_id: FileID,
    },
    GenericParsing {
        location: FileLocation,
        file_id: FileID,
        msg: String,
    },
    FileOs {
        path: String,
    },
    NoMain,
    MultipleMain,
    CompilerVersion {
        path: String,
        required_version: Version,
        version: Version,
    },
    MissingSemicolon{
        location: FileLocation,
        file_id: FileID,
    },
    UnrecognizedInclude {
        location: FileLocation,
        file_id: FileID,
    },
    UnrecognizedVersion {
        location: FileLocation,
        file_id: FileID,
    },
    UnrecognizedPragma {
        location: FileLocation,
        file_id: FileID,
    },
    IllegalExpression {
        location: FileLocation,
        file_id: FileID,
    },
}

impl Error {
    pub fn produce_report(self) -> Report {
        match self {
            Error::UnclosedComment { location, file_id } => {
                let mut report =
                    Report::error("unterminated /* */".to_string(), ReportCode::ParseFail);
                report.add_primary(location, file_id, "Comment starts here".to_string());
                report
            }
            Error::GenericParsing {
                location,
                file_id,
                msg,
            } => {
                let mut report = Report::error(msg, ReportCode::ParseFail);
                report.add_primary(location, file_id, "Invalid syntax".to_string());
                report
            }
            Error::FileOs { path } => Report::error(
                format!("Could not open file {}", path),
                ReportCode::ParseFail,
            ),
            Error::NoMain => Report::error(
                "No main specified in the project structure".to_string(),
                ReportCode::NoMainFoundInProject,
            ),
            Error::MultipleMain =>{
                Report::error(
                    "Multiple main components in the project structure".to_string(),
                    ReportCode::MultipleMainInComponent,
                )
            }
            Error::CompilerVersion {
                path,
                required_version,
                version,
            } => {
                Report::error(
                    format!("File {} requires pragma version {:?} that is not supported by the compiler (version {:?})", path, required_version, version ),
                    ReportCode::CompilerVersionError,
                )
            }
            Error::MissingSemicolon {
                location,
                file_id,
            } => {
                let mut report = Report::error(format!("Missing semicolon"), 
                    ReportCode::ParseFail);
                report.add_primary(location, file_id, "A semicolon is needed here".to_string());
                report
            }
            Error::UnrecognizedInclude{location, file_id} => {
                let mut report =
                Report::error("unrecognized argument in include directive".to_string(), ReportCode::ParseFail);
            report.add_primary(location, file_id, "this argument".to_string());
            report

            }
            Error::UnrecognizedPragma{location, file_id} => {
                let mut report =
                Report::error("unrecognized argument in pragma directive".to_string(), ReportCode::ParseFail);
            report.add_primary(location, file_id, "this argument".to_string());
            report

            }        
            Error::UnrecognizedVersion{location, file_id} => {
                let mut report =
                Report::error("unrecognized version argument in pragma directive".to_string(), ReportCode::ParseFail);
            report.add_primary(location, file_id, "this argument".to_string());
            report
            }      
            Error::IllegalExpression{location, file_id} => {
                let mut report =
                Report::error("illegal expression".to_string(), ReportCode::ParseFail);
            report.add_primary(location, file_id, "here".to_string());
            report
            }     
        }
    }
}

pub struct NoCompilerVersionWarning {
    pub path: String,
    pub version: Version,
}

impl NoCompilerVersionWarning {
    pub fn produce_report(error: Self) -> Report {
        Report::warning(
            format!(
                "File {} does not include pragma version. Assuming pragma version {:?}",
                error.path, error.version
            ),
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