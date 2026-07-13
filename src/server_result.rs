use crate::resp::RESP;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    CommandError,
    IncorrectData,
    StorageNotInititalised,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CommandError => write!(f, "Command error"),
            ServerError::IncorrectData => write!(f, "Incorrect data"),
            ServerError::StorageNotInititalised => write!(f, "Storage not initialised"),
        }
    }
}
#[derive(Debug)]
pub enum ServerValue {
    RESP(RESP),
}
pub type ServerResult = Result<ServerValue, ServerError>;
#[derive(Debug)]
pub enum ServerMessage {
    Data(ServerValue),
    Error(ServerError),
}
