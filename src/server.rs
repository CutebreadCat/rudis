use crate::RESP;
use crate::commands::{echo, ping,set,get};
use crate::connection::ConnectionMessage;
use crate::request::Request;
use crate::server_result::{ServerError, ServerMessage, ServerValue};
use crate::storage::Storage;
use crate::storage_result::{StorageError, StorageResult};
use std::time::Duration;
use std::{
    fmt,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

pub struct Server {
    pub storage: Option<Storage>,
}
impl Server {
    pub fn new(storage: Storage) -> Self {
        Server {
            storage: Some(storage),
        }
    }
    pub fn set_storage(&mut self, storage: Storage) {
        self.storage = Some(storage);
    }
    pub fn expire_keys(&mut self) {
        let storage = match self.storage.as_mut() {
            Some(storage) => storage,
            None => return,
        };
        storage.expire_keys();
    }
}

pub async fn run_server(mut server: Server, mut crx: mpsc::Receiver<ConnectionMessage>) {
    let mut interval_timer = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            Some(message) = crx.recv()=>{
                match message{
                    ConnectionMessage::Request(request) => {
                        process_requeset(request,&mut server).await;
                    }
                }
            }
           _ = interval_timer.tick()=>{
               server.expire_keys();
           }
        }
    }
}

pub async fn process_requeset(request: Request, server: &mut Server) {
    let elements = match &request.resp {
        RESP::Array(v) => v,
        _ => {
            request.error(ServerError::IncorrectData).await;
            return;
        }
    };

    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.clone()),
            _ => {
                request.error(ServerError::IncorrectData).await;
                return;
            } //这边不应该直接返回对于一个正确的设计应该是返回一个报错但不终止客户端的连接
        }
    }
    let command_name = command[0].to_lowercase();
    match command_name.as_str() {
        "echo"=>{
            echo::command(server, &request,&command).await;
        }
        "ping"=>{
            ping::command(server, &request,&command).await;
        }
        "set"=>{
            set::command(server, &request,&command).await;
        }
        "get"=>{
            get::command(server, &request,&command).await;
        }
        "info"=>{
            request.data(ServerValue::RESP(RESP::SimpleString("TODO".to_string()))).await;
        }

        _ => {
            request.error(ServerError::CommandInternalError(command_name)).await;
        }
    }
}
