use program_structure::error_definition::Report;
use std::collections::HashSet;
use std::path::{Component, PathBuf};

// Replacement for std::fs::canonicalize that doesn't verify the path exists
// Plucked from https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
// Advice from https://www.reddit.com/r/rust/comments/hkkquy/comment/fwtw53s/?utm_source=share&utm_medium=web2x&context=3
fn normalize_path(path: &PathBuf) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

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
        // Replaced `std::fs::canonicalize` with a custom `normalize_path` function.
        // No existence testing in wasm
        let path = normalize_path(&crr);
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
