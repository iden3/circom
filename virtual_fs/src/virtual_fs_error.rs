#[derive(Debug)]
pub enum VirtualFsError {
    IoError(std::io::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    ToUtf8Error,
    CwdInvalidError,
}

impl From<std::io::Error> for VirtualFsError {
    fn from(err: std::io::Error) -> Self {
        VirtualFsError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for VirtualFsError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        VirtualFsError::FromUtf8Error(err)
    }
}
