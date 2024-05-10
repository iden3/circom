use std::path::PathBuf;

/**
 * A wrapper around PathBuf that exposes only path manipulation tools,
 * providing a more clear separation from the filesystem.
 * 
 * (Eg path.exists() is not available.)
 */
#[derive(Debug)]
pub struct SimplePath(pub PathBuf);

impl SimplePath {
    pub fn new(path: &str) -> Self {
        SimplePath(PathBuf::from(path))
    }

    pub fn join(&self, path: &str) -> Self {
        SimplePath(self.0.join(path))
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(|p| SimplePath(p.to_path_buf()))
    }

    pub fn push(&mut self, path: &str) {
        self.0.push(path);
    }

    pub fn file_stem(&self) -> Option<String> {
        self.0.file_stem().map(|s| s.to_string_lossy().to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.to_string_lossy().to_string()
    }
}

impl From<&str> for SimplePath {
    fn from(path: &str) -> Self {
        SimplePath::new(path)
    }
}

impl From<String> for SimplePath {
    fn from(path: String) -> Self {
        SimplePath::new(&path)
    }
}
