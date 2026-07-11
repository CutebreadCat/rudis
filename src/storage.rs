use std::collections::HashMap;

use crate::resp::RESP;
use crate::storage_result::{StorageError, StorageResult};

#[derive(Debug, PartialEq, Eq)]
pub enum StorageValue{
    String(String),
}
pub struct Storage{
    store: HashMap<String, StorageValue>,
}
impl Storage{
    pub fn new() -> Self{
        let stroe:HashMap<String, StorageValue> = HashMap::new();
        Self{store:stroe}
    }

    pub fn process_command(&mut self, command: Vec<String>)->StorageResult<RESP>{
        match command[0].to_lowercase().as_str() {
            "ping" => self.command_ping(command),
            "echo" => self.command_echo(command),
            "info" => Ok(RESP::BulkString(String::from(
            "# Server\r\nredis_version:7.0.0\r\n",
           ))),
        "quit"=>Ok(RESP::SimpleString(String::from("OK"))),
         "set"=> self.command_set(command),
         "get"=> self.command_get(command), 
            _ => Err(StorageError::CommandNotAvaliable(command[0].clone())),
         
        }
        
        
    }
    fn command_ping (&mut self, command: Vec<String>)->StorageResult<RESP>{
        Ok(RESP::SimpleString("PONG".to_string()))
    }
    fn command_echo (&mut self, command: Vec<String>)->StorageResult<RESP>{
        Ok(RESP::SimpleString(command[1].clone()))
    }
    fn set(&mut self, key: String, value: String) -> StorageResult<String> {
        self.store.insert(key, StorageValue::String(value));
        Ok(("OK".to_string()))
    }
    fn get(&mut self, key: String) -> StorageResult<Option<String>> {
        let value = self.store.get(&key);
        match value {
            Some(StorageValue::String(v)) => return Ok(Some(v.clone())),
            None => return Ok(None),
        }
    }
    fn command_set(&mut self, command: Vec<String>)->StorageResult<RESP>{
        if command.len() != 3 {
            return Err(StorageError::IncorrectRequest);
        }
        let _ = self.set(command[1].clone(), command[2].clone());
        Ok(RESP::SimpleString("OK".to_string()))
    }
    fn command_get(&mut self, command: Vec<String>)->StorageResult<RESP>{
        if command.len() != 2 {
            return Err(StorageError::IncorrectRequest);
        }
        let output = self.get(command[1].clone());
        match output{
            Ok(Some(v))=>Ok(RESP::BulkString(v)),
            Ok(None)=>Ok(RESP::Null),
            Err(_)=>Err(StorageError::CommandNotAvaliable(command.join(" "))),
        }
    }                                                                                                                                                                
}

