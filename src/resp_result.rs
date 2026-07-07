use std::fmt::{self};
use std::string::FromUtf8Error;
use std::num;
#[derive(Debug, PartialEq, Eq)]
pub enum RESPError {
    FromUtf8,
    IncorrectLength(RESPLenth),
    OutOfBounds(usize),
    WrongType,
    ParseInt,
    Unkonwn,
}
impl fmt::Display for RESPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RESPError::OutOfBounds(index) => write!(f, "Out of bounds at index {}", index),
            RESPError::FromUtf8 => write!(f, "Cannot convert from UTF-8"),
            RESPError::WrongType => write!(f, "Wrong type"),
            RESPError::Unkonwn => write!(f, "Unkonwn error"),
            RESPError::ParseInt => write!(f, "Cannot parse int"),
            RESPError::IncorrectLength(length) => write!(f, "Incorrect length: {}", length),
        }
    }
}
impl From<FromUtf8Error> for RESPError {
    fn from(_err: FromUtf8Error) -> Self {
        Self::FromUtf8
    }
}

impl From<num::ParseIntError> for RESPError {
    fn from(_err: num::ParseIntError) -> Self {
        Self::ParseInt
    }
}

pub type RESPResult<T> = Result<T, RESPError>;

pub type RESPLenth = i32;