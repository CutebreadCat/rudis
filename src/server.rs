use crate::{RESP, resp};
use crate::commands::{echo, ping,set,get,info};
use crate::connection::ConnectionMessage;
use crate::request::Request;
use crate::server_result::{ServerError, ServerResult, ServerValue};
use crate::storage::Storage;
use crate::resp::bytes_to_resp;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    
};

use std::time::Duration;
use tokio::sync::mpsc;
use crate::replication::ReplicationConfig;

pub struct Server {
    pub info: Serverinfo,
    pub storage: Option<Storage>,
    pub replication:ReplicationConfig

}
impl Server {
    pub fn new(host: String, port: u16) -> Self {
        Server {
            info: Serverinfo { host, port },
            storage: None,
            replication: ReplicationConfig::new_master(),
        }
    }
    pub fn set_storage(&mut self, storage: Storage) {
        self.storage = Some(storage);
    }
    pub fn set_replication(&mut self, replication:ReplicationConfig) {
        self.replication = replication;
    }
    pub fn expire_keys(&mut self) {
        let storage = match self.storage.as_mut() {
            Some(storage) => storage,
            None => return,
        };
        storage.expire_keys();
    }
}
pub struct Serverinfo{
    pub host:String,
    pub port:u16,
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
            info::command(server, &request,&command).await;
        }

        _ => {
            request.error(ServerError::CommandInternalError(command_name)).await;
        }
    }
}

pub async fn handshake(stream:&mut TcpStream) -> ServerResult{
    let ping = RESP::Array(vec![RESP::BulkString(String::from("PING"))]);
    stream.write_all(ping.to_string().as_bytes()).await.map_err(|e| ServerError::HandshakeFailed(
        format!("Sending{} - Cannot write to stream: {}", ping.to_string(), e.to_string())
    ))?;
    let mut buffer = [0;512];
    let size = stream.read(&mut buffer).await.map_err(|e| ServerError::HandshakeFailed(
        format!("Receiving - Cannot read from stream: {}", e.to_string())
    ))?;
    if size == 0 {
        return Err(ServerError::HandshakeFailed("Receiving - No data received".to_string()));
    }
    let mut index:usize = 0;
    let resp = bytes_to_resp(&buffer, &mut index).map_err(|e|{
        ServerError::HandshakeFailed(format!(
            "Sending {} -Wrong server answer: {}",ping.to_string(),e.to_string()
        ))
    })?;

    if resp != RESP::SimpleString(String::from("PONG")) {
        return Err(ServerError::HandshakeFailed(format!(
            "Sending {} -Wrong server answer: {}",ping.to_string(),resp.to_string()
        )));
    }
    
    Ok(ServerValue::None)
}