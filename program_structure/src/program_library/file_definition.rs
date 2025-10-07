use codespan_reporting::files::{Files, SimpleFile, SimpleFiles};
use std::ops::Range;

pub type FileSource = String;
pub type FilePath = String;
pub type FileID = usize;
pub type FileLocation = Range<usize>;
type FileStorage = SimpleFiles<FilePath, FileSource>;

#[derive(Clone, Debug)]
pub struct FileLibrary {
    files: FileStorage,
}

impl Default for FileLibrary {
    fn default() -> Self {
        let mut files = FileStorage::new();
        files.add(String::from("<generated>"), String::from(""));
        FileLibrary { files }
    }
}

impl FileLibrary {
    pub fn new() -> FileLibrary {
        FileLibrary::default()
    }
    pub fn add_file(&mut self, file_name: FilePath, file_source: FileSource) -> FileID {
        self.get_mut_files().add(file_name, file_source)
    }
    pub fn get_line(&self, start: usize, file_id: FileID) -> Option<usize> {
        self.files.location(file_id, start).map(|loc| loc.line_number)
    }
    pub fn get_column(&self, start: usize, file_id: FileID) -> Option<usize> {
        self.files.location(file_id, start).map(|loc| loc.column_number)
    }
    pub fn get_file(&self, file_id: &FileID) -> Option<&SimpleFile<FilePath, FileSource>> {
        self.files.get(*file_id)
    }
    pub fn get_filename_or(&self, file_id: &FileID, default: &FilePath) -> FilePath {
        let sf_opt = self.get_file(file_id);
        match sf_opt {
            Some(sf) => sf.name().replace("\"", ""),
            None => default.clone(),
        }
    }

    pub fn get_filename_or_default(&self, file_id: &FileID) -> FilePath {
        self.get_filename_or(file_id, &FilePath::from("<unknown>"))
    }
    pub fn to_storage(&self) -> &FileStorage {
        &self.get_files()
    }
    fn get_files(&self) -> &FileStorage {
        &self.files
    }
    fn get_mut_files(&mut self) -> &mut FileStorage {
        &mut self.files
    }
}
pub fn generate_file_location(start: usize, end: usize) -> FileLocation {
    start..end
}
