use std::collections::BTreeMap;

use crate::{v_path::VPath, virtual_fs_error::VirtualFsError, virtual_fs_result::VirtualFsResult, FileSystem};

pub struct MemoryFs {
    cwd: VPath,
    files: BTreeMap<String, Vec<u8>>,
}

impl MemoryFs {
    pub fn new<P: Into<VPath>>(cwd: P) -> Self {
        MemoryFs {
            cwd: cwd.into(),
            files: BTreeMap::new(),
        }
    }
}

impl FileSystem for MemoryFs {
    fn cwd(&self) -> VirtualFsResult<VPath> {
        Ok(self.cwd.clone())
    }

    fn set_cwd<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()> {
        self.cwd = path.into();
        Ok(())
    }

    fn read<P: Into<VPath>>(&self, path: P) -> VirtualFsResult<Vec<u8>> {
        let path = self.normalize(path)?.to_string();
        
        self.files
            .get(&path)
            .cloned()
            .ok_or(VirtualFsError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path),
            )))
    }

    fn write<P: Into<VPath>>(&mut self, path: P, data: &[u8]) -> VirtualFsResult<()> {
        let path = self.normalize(path)?.to_string();
        self.files.insert(path, data.into());
        Ok(())
    }

    fn create_dir<P: Into<VPath>>(&mut self, _path: P) -> VirtualFsResult<()> {
        // MemoryFs doesn't actually use directories
        Ok(())
    }

    fn create_dir_all<P: Into<VPath>>(&mut self, _path: P) -> VirtualFsResult<()> {
        // MemoryFs doesn't actually use directories
        Ok(())
    }

    fn rimraf<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()> {
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
}
