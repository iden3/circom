use std::{fs::File, io::{Read, Write}, path::PathBuf};

use crate::{file_system::FileSystem, v_path::VPath, fs_result::FsResult};

pub struct RealFs {}

impl RealFs {
    pub fn new() -> Self {
        RealFs {}
    }
}

impl FileSystem for RealFs {
    fn cwd(&self) -> FsResult<VPath> {
        let cwd = std::env::current_dir()?;
        Ok(VPath(cwd))
    }

    fn set_cwd(&mut self, path: &VPath) -> FsResult<()> {
        std::env::set_current_dir(&path.0)?;

        Ok(())
    }

    fn exists(&self, path: &VPath) -> FsResult<bool> {
        Ok(path.0.exists())
    }

    fn read(&self, path: &VPath) -> FsResult<Vec<u8>> {
        let mut file = File::open(&path.0)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    fn write(&mut self, path: &VPath, data: &[u8]) -> FsResult<()> {
        let mut file = File::create(&path.0)?;
        file.write_all(data)?;

        Ok(())
    }

    fn create_dir(&mut self, path: &VPath) -> FsResult<()> {
        std::fs::create_dir_all(&path.0)?;

        Ok(())
    }

    fn create_dir_all(&mut self, path: &VPath) -> FsResult<()> {
        std::fs::create_dir_all(&path.0)?;

        Ok(())
    }

    fn rimraf(&mut self, path: &VPath) -> FsResult<()> {
        let path: &PathBuf = &path.0;

        if path.parent().is_none() {
            panic!("Refused `rm -rf /` catastrophe");
        }

        if !path.exists() {
            return Ok(());
        }

        match path.metadata()?.is_dir() {
            true => {
                for entry in path.read_dir()? {
                    self.rimraf(&VPath(entry?.path()))?;
                }

                std::fs::remove_dir(path)?;
            }
            false => {
                std::fs::remove_file(path)?;
            }
        };

        Ok(())
    }

    fn remove_file(&mut self, path: &VPath) -> FsResult<()> {
        std::fs::remove_file(&path.0)?;

        Ok(())
    }
}
