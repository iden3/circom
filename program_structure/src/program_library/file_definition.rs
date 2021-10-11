use codespan_reporting::files::{Files, SimpleFiles};
use std::ops::Range;

pub type FileSource = String;
pub type FilePath = String;
pub type FileID = usize;
pub type FileLocation = Range<usize>;
type FileStorage = SimpleFiles<FilePath, FileSource>;

#[derive(Clone)]
pub struct FileLibrary {
    files: FileStorage,
}

impl Default for FileLibrary {
    fn default() -> Self {
        FileLibrary { files: FileStorage::new() }
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
        match self.files.line_index(file_id, start) {
            Some(lines) => Some(lines + 1),
            None => None,
        }
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
