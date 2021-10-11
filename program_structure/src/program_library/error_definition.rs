use super::error_code::ReportCode;
use super::file_definition::{FileID, FileLibrary, FileLocation};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term;

pub type ReportCollection = Vec<Report>;
pub type DiagnosticCode = String;
type ReportLabel = Label<FileID>;
type ReportNote = String;

#[derive(Copy, Clone)]
enum MessageCategory {
    Error,
    Warning,
}
impl MessageCategory {
    fn is_error(&self) -> bool {
        use MessageCategory::*;
        match self {
            Error => true,
            _ => false,
        }
    }
    fn is_warning(&self) -> bool {
        use MessageCategory::*;
        match self {
            Warning => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Report {
    category: MessageCategory,
    error_message: String,
    error_code: ReportCode,
    primary: Vec<ReportLabel>,
    secondary: Vec<ReportLabel>,
    notes: Vec<ReportNote>,
}
impl Report {
    fn new(category: MessageCategory, error_message: String, error_code: ReportCode) -> Report {
        Report {
            category,
            error_message,
            error_code,
            primary: Vec::new(),
            secondary: Vec::new(),
            notes: Vec::new(),
        }
    }
    pub fn print_reports(reports: &[Report], file_library: &FileLibrary) {
        use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = term::Config::default();
        let mut diagnostics = Vec::new();
        let files = file_library.to_storage();
        for report in reports.iter() {
            diagnostics.push(report.to_diagnostic());
        }
        for diagnostic in diagnostics.iter() {
            let print_result = term::emit(&mut writer.lock(), &config, files, &diagnostic);
            if print_result.is_err() {
                panic!("Error printing reports")
            }
        }
    }
    fn error_code_to_diagnostic_code(error_code: &ReportCode) -> DiagnosticCode {
        error_code.to_string()
    }
    pub fn error(error_message: String, code: ReportCode) -> Report {
        Report::new(MessageCategory::Error, error_message, code)
    }
    pub fn warning(error_message: String, code: ReportCode) -> Report {
        Report::new(MessageCategory::Warning, error_message, code)
    }
    pub fn add_primary(
        &mut self,
        location: FileLocation,
        file_id: FileID,
        message: String,
    ) -> &mut Self {
        let label = ReportLabel::primary(file_id, location).with_message(message);
        self.get_mut_primary().push(label);
        self
    }
    pub fn add_secondary(
        &mut self,
        location: FileLocation,
        file_id: FileID,
        possible_message: Option<String>,
    ) -> &mut Self {
        let mut label = ReportLabel::secondary(file_id, location);
        if let Option::Some(message) = possible_message {
            label = label.with_message(message);
        }
        self.get_mut_secondary().push(label);
        self
    }
    pub fn add_note(&mut self, note: String) -> &mut Self {
        self.get_mut_notes().push(note);
        self
    }

    fn to_diagnostic(&self) -> Diagnostic<FileID> {
        let mut labels = self.get_primary().clone();
        let mut secondary = self.get_secondary().clone();
        labels.append(&mut secondary);

        if self.is_warning() { Diagnostic::warning() } else { Diagnostic::error() }
            .with_message(self.get_message())
            .with_code(Report::error_code_to_diagnostic_code(self.get_code()))
            .with_labels(labels)
            .with_notes(self.get_notes().clone())
    }

    pub fn is_error(&self) -> bool {
        self.get_category().is_error()
    }
    pub fn is_warning(&self) -> bool {
        self.get_category().is_warning()
    }
    fn get_category(&self) -> &MessageCategory {
        &self.category
    }
    fn get_message(&self) -> &String {
        &self.error_message
    }
    fn get_code(&self) -> &ReportCode {
        &self.error_code
    }
    fn get_primary(&self) -> &Vec<ReportLabel> {
        &self.primary
    }
    fn get_mut_primary(&mut self) -> &mut Vec<ReportLabel> {
        &mut self.primary
    }
    fn get_secondary(&self) -> &Vec<ReportLabel> {
        &self.secondary
    }
    fn get_mut_secondary(&mut self) -> &mut Vec<ReportLabel> {
        &mut self.secondary
    }
    fn get_notes(&self) -> &Vec<ReportNote> {
        &self.notes
    }
    fn get_mut_notes(&mut self) -> &mut Vec<ReportNote> {
        &mut self.notes
    }
}
