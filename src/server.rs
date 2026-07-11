use std::{fmt, sync::{Arc, Mutex}};
use crate::RESP;
use crate::storage::{Storage};
use crate::storage_result::{StorageError, StorageResult};




pub fn process_requeset(request: RESP, storage: Arc<Mutex<Storage>>) -> StorageResult<RESP> {
    let elements = match request {
        RESP::Array(v) => v,
        _ => return Err(StorageError::IncorrectRequest),
    };

    if elements.is_empty() {
        return Ok(RESP::Error(String::from("ERR wrong number of arguments")));
    }

    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.clone()),
            _ => return Err(StorageError::IncorrectRequest),
            //这边不应该直接返回对于一个正确的设计应该是返回一个报错但不终止客户端的连接
        }
    }


    let mut guard = storage.lock().unwrap();
    let reponse = guard.process_command(command);
    reponse
}
