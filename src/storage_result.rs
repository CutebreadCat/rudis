use std::fmt;
#[derive(Debug)]
pub enum StorageError {
   IncorrectRequest,
    CommandNotAvaliable(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::IncorrectRequest => write!(f, "IncorrectRequest"),
            StorageError::CommandNotAvaliable(command) => write!(f, "CommandNotAvaliable: {}", command),
        }
    }
}
pub type StorageResult<T> = Result<T, StorageError>;