use std::fmt;
use crate::RESP;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    CommandError,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CommandError => write!(f, "Command error"),
        }
    }
}

pub type ServerResult<T> = Result<T, ServerError>;

pub fn process_requeset(request: RESP) -> ServerResult<RESP> {
    let elements = match request {
        RESP::Array(v) => v,
        _ => return Err(ServerError::CommandError),
    };

    if elements.is_empty() {
        return Ok(RESP::Error(String::from("ERR wrong number of arguments")));
    }

    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.to_string()),
            _ => return Ok(RESP::Error(String::from("ERR unknown command"))),
        }
    }

    println!("Received command: {:?}", command);

    match command[0].to_lowercase().as_str() {
        "ping" => Ok(RESP::SimpleString(String::from("PONG"))),
        "command" => Ok(RESP::Array(vec![])),
        "info" => Ok(RESP::BulkString(String::from(
            "# Server\r\nredis_version:7.0.0\r\n",
        ))),
        _ => Ok(RESP::Error(String::from("ERR unknown command"))),
    }
}
