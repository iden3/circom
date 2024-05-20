use std::path::PathBuf;

use crate::{fs_error::FsError, fs_result::FsResult};

/**
 * A wrapper around PathBuf that provides a more clear separation from the
 * filesystem. Most IO methods are removed and some are exposed with the real_
 * prefix.
 */
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VPath(pub PathBuf);

impl VPath {
    pub fn new(path: &str) -> Self {
        VPath(PathBuf::from(path))
    }

    pub fn join(&self, path: &str) -> Self {
        VPath(self.0.join(path))
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(|p| VPath(p.to_path_buf()))
    }

    pub fn push(&mut self, path: &str) {
        self.0.push(path);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn set_extension(&mut self, ext: &str) {
        self.0.set_extension(ext);
    }

    pub fn file_stem(&self) -> Option<String> {
        self.0.file_stem().map(|s| s.to_string_lossy().to_string())
    }

    /**
     * Returns the normal form of the path, relative to the given cwd.
     *
     * Normalization also means:
     * - Removing empty components
     * - Removing "." components
     * - Removing ".." components and their parent
     *
     * Does not follow symlinks.
     */
    pub fn normalize(&self, cwd: &VPath) -> FsResult<Self> {
        if !cwd.0.has_root() {
            return Err(FsError::CwdInvalidError);
        }

        let mut combined_path = cwd.0.clone();
        combined_path.push(&self.0);

        let mut normal_path = PathBuf::new();

        for part in combined_path.components() {
            match part {
                std::path::Component::Prefix(_)
                | std::path::Component::RootDir => {
                    normal_path.push(part);
                }
                std::path::Component::Normal(os_str) => {
                    if !os_str.is_empty() {
                        normal_path.push(part);
                    }
                }
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    normal_path.pop();
                }
            }
        }

        Ok(VPath(normal_path))
    }

    pub fn real_canonicalize(&self) -> FsResult<Self> {
        Ok(VPath(self.0.canonicalize()?))
    }

    pub fn real_exists(&self) -> bool {
        self.0.exists()
    }

    pub fn real_is_file(&self) -> bool {
        self.0.is_file()
    }

    pub fn real_is_dir(&self) -> bool {
        self.0.is_dir()
    }
}

impl From<&str> for VPath {
    fn from(path: &str) -> Self {
        VPath::new(path)
    }
}

impl From<String> for VPath {
    fn from(path: String) -> Self {
        VPath::new(&path)
    }
}

impl std::fmt::Display for VPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}
