use std::collections::BTreeMap;

use crate::{v_path::VPath, fs_error::FsError, fs_result::FsResult, FileSystem};

pub struct MemoryFs {
    cwd: VPath,
    files: BTreeMap<String, Vec<u8>>,
}

impl MemoryFs {
    pub fn new(cwd: VPath) -> Self {
        MemoryFs {
            cwd,
            files: BTreeMap::new(),
        }
    }
}

impl FileSystem for MemoryFs {
    fn cwd(&self) -> FsResult<VPath> {
        Ok(self.cwd.clone())
    }

    fn set_cwd(&mut self, path: &VPath) -> FsResult<()> {
        self.cwd = path.clone();
        Ok(())
    }

    fn exists(&self, path: &VPath) -> FsResult<bool> {
        let path = self.normalize(path)?.to_string();
        Ok(self.files.contains_key(&path))
    }

    fn read(&self, path: &VPath) -> FsResult<Vec<u8>> {
        let path = self.normalize(path)?.to_string();

        self.files
            .get(&path)
            .cloned()
            .ok_or(FsError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path),
            )))
    }

    fn write(&mut self, path: &VPath, data: &[u8]) -> FsResult<()> {
        let path = self.normalize(path)?.to_string();
        self.files.insert(path, data.into());
        Ok(())
    }

    fn create_dir(&mut self, _path: &VPath) -> FsResult<()> {
        // MemoryFs doesn't actually use directories
        Ok(())
    }

    fn create_dir_all(&mut self, _path: &VPath) -> FsResult<()> {
        // MemoryFs doesn't actually use directories
        Ok(())
    }

    fn rimraf(&mut self, path: &VPath) -> FsResult<()> {
        let path = self.normalize(path)?.to_string();

        if path == "" || path == "/" {
            panic!("Refused `rm -rf /` catastrophe");
        }

        let (file_path, dir_path) = if path.ends_with('/') {
            let mut file_path = path.to_string();
            file_path.pop();

            (file_path, path.to_string())
        } else {
            (path.to_string(), path.to_string() + "/")
        };

        let mut paths_to_remove = vec![file_path];

        for (p, _) in self.files.range(dir_path.clone()..) {
            if p.starts_with(&dir_path) {
                paths_to_remove.push(p.clone());
            } else {
                break;
            }
        }

        for p in paths_to_remove {
            self.files.remove(&p);
        }

        Ok(())
    }

    fn remove_file(&mut self, path: &VPath) -> FsResult<()> {
        let path = self.normalize(path)?.to_string();
        self.files.remove(&path);
        Ok(())
    }
}
