use super::errors::FileOsError;
use program_structure::error_definition::Report;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct FileStack {
    current_location: PathBuf,
    black_paths: HashSet<PathBuf>,
    stack: Vec<PathBuf>,
}

impl FileStack {
    pub fn new(src: PathBuf) -> FileStack {
        let mut location = src.clone();
        location.pop();
        FileStack { current_location: location, black_paths: HashSet::new(), stack: vec![src] }
    }

    pub fn add_include(f_stack: &mut FileStack, path: String) -> Result<(), Report> {
        let mut crr = f_stack.current_location.clone();
        crr.push(path.clone());
        let path = std::fs::canonicalize(crr)
            .map_err(|_| FileOsError { path: path.clone() })
            .map_err(|e| FileOsError::produce_report(e))?;
        if !f_stack.black_paths.contains(&path) {
            f_stack.stack.push(path);
        }
        Ok(())
    }

    pub fn take_next(f_stack: &mut FileStack) -> Option<PathBuf> {
        loop {
            match f_stack.stack.pop() {
                None => {
                    break None;
                }
                Some(file) if !f_stack.black_paths.contains(&file) => {
                    f_stack.current_location = file.clone();
                    f_stack.current_location.pop();
                    f_stack.black_paths.insert(file.clone());
                    break Some(file);
                }
                _ => {}
            }
        }
    }
}
