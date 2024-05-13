use std::{fs::File, io::{Read, Write}, path::PathBuf};

use crate::{file_system::FileSystem, v_path::VPath, virtual_fs_result::VirtualFsResult};

pub struct RealFs {}

impl RealFs {
    pub fn new() -> Self {
        RealFs {}
    }
}

impl FileSystem for RealFs {
    fn cwd(&self) -> VirtualFsResult<VPath> {
        let cwd = std::env::current_dir()?;
        Ok(VPath(cwd))
    }

    fn set_cwd<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()> {
        let path = path.into().0;
        std::env::set_current_dir(path)?;

        Ok(())
    }

    fn read<P: Into<VPath>>(&self, path: P) -> VirtualFsResult<Vec<u8>> {
        let mut file = File::open(path.into().0)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    fn write<P: Into<VPath>>(&mut self, path: P, data: &[u8]) -> VirtualFsResult<()> {
        let mut file = File::create(path.into().0)?;
        file.write_all(data)?;

        Ok(())
    }

    fn create_dir<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()> {
        let path = path.into().0;
        std::fs::create_dir_all(path)?;

        Ok(())
    }

    fn create_dir_all<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()> {
        let path = path.into().0;
        std::fs::create_dir_all(path)?;

        Ok(())
    }

    fn rimraf<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()> {
        let path: PathBuf = path.into().0;

        if path.parent().is_none() {
            panic!("Refused `rm -rf /` catastrophe");
        }

        if !path.exists() {
            return Ok(());
        }

        match path.metadata()?.is_dir() {
            true => {
                for entry in path.read_dir()? {
                    let path = entry?.path();
                    let child = path.to_str().unwrap();
                    self.rimraf(child)?;
                }

                std::fs::remove_dir(path)?;
            }
            false => {
                std::fs::remove_file(path)?;
            }
        };

        Ok(())
    }
}
