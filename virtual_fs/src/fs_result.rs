use crate::fs_error::FsError;

pub type FsResult<T> = Result<T, FsError>;
