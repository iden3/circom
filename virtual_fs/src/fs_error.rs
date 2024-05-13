#[derive(Debug)]
pub enum FsError {
    IoError(std::io::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    ToUtf8Error,
    CwdInvalidError,
    Unknown,
}

impl FsError {
    // TODO: Update return type of users instead?
    pub fn into_io_error(self) -> std::io::Error {
        match self {
            FsError::IoError(err) => err,
            _ => std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", self)),
        }
    }
}

impl From<std::io::Error> for FsError {
    fn from(err: std::io::Error) -> Self {
        FsError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for FsError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        FsError::FromUtf8Error(err)
    }
}
