use crate::virtual_fs_error::VirtualFsError;

pub type VirtualFsResult<T> = Result<T, VirtualFsError>;
