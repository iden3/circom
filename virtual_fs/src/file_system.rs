use crate::{v_path::VPath, fs_result::FsResult};

pub trait FileSystem {
    fn cwd(&self) -> FsResult<VPath>;
    fn set_cwd(&mut self, path: &VPath) -> FsResult<()>;

    fn normalize(&self, path: &VPath) -> FsResult<VPath> {
        let cwd = self.cwd()?;
        Ok(path.normalize(&cwd)?)
    }

    fn exists(&self, path: &VPath) -> FsResult<bool>;
    fn read(&self, path: &VPath) -> FsResult<Vec<u8>>;
    fn write(&mut self, path: &VPath, data: &[u8]) -> FsResult<()>;

    fn read_string(&self, path: &VPath) -> FsResult<String> {
        let data = self.read(path)?;
        Ok(String::from_utf8(data)?)
    }

    fn create_dir(&mut self, path: &VPath) -> FsResult<()>;
    fn create_dir_all(&mut self, path: &VPath) -> FsResult<()>;
    fn rimraf(&mut self, path: &VPath) -> FsResult<()>;
    fn remove_file(&mut self, path: &VPath) -> FsResult<()>;
}
