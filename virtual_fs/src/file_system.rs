use crate::{v_path::VPath, virtual_fs_result::VirtualFsResult};

pub trait FileSystem {
    fn cwd(&self) -> VirtualFsResult<VPath>;
    fn set_cwd<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()>;

    fn normalize<P: Into<VPath>>(&self, path: P) -> VirtualFsResult<VPath> {
        let cwd = self.cwd()?;
        Ok(path.into().normalize(&cwd)?)
    }

    fn read<P: Into<VPath>>(&self, path: P) -> VirtualFsResult<Vec<u8>>;
    fn write<P: Into<VPath>>(&mut self, path: P, data: &[u8]) -> VirtualFsResult<()>;

    fn read_string<P: Into<VPath>>(&self, path: P) -> VirtualFsResult<String> {
        let data = self.read(path)?;
        Ok(String::from_utf8(data)?)
    }

    fn create_dir<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()>;
    fn create_dir_all<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()>;
    fn rimraf<P: Into<VPath>>(&mut self, path: P) -> VirtualFsResult<()>;
}
