use crate::resp::RESP;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    CommandInternalError(String),
    CommandSyntaxError(String),
    IncorrectData,
    StorageNotInititalised,
    HandshakeFailed(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CommandInternalError(s) => write!(f, "Internal error: {}", s),
            ServerError::CommandSyntaxError(s) => write!(f, "Syntax error: {}", s),
            ServerError::IncorrectData => write!(f, "Incorrect data"),
            ServerError::StorageNotInititalised => write!(f, "Storage not initialised"),
            ServerError::HandshakeFailed(s) => write!(f, "Handshake failed: {}", s),
        }
    }
}
#[derive(Debug)]
pub enum ServerValue {
    RESP(RESP),
    Binary(Vec<u8>),
    None,
}
pub type ServerResult = Result<ServerValue, ServerError>;
#[derive(Debug)]
pub enum ServerMessage {
    Data(ServerValue),
    Error(ServerError),
}
