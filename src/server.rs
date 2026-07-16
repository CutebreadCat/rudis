use crate::commands::{echo, get, info, ping, psync, replonf, set};
use crate::connection::{ConnectionMessage, stream_receive_resp, streams_send_receive_resp};
use crate::request::Request;
use crate::server_result::{ServerError, ServerResult, ServerValue};
use crate::storage::Storage;
use crate::{RESP, resp};
use tokio::net::TcpStream;

use crate::replication::ReplicationConfig;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct Server {
    pub info: Serverinfo,
    pub storage: Option<Storage>,
    pub replication: ReplicationConfig,
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
    pub fn set_replication(&mut self, replication: ReplicationConfig) {
        self.replication = replication;
    }
    pub fn expire_keys(&mut self) {
        let storage = match self.storage.as_mut() {
            Some(storage) => storage,
            None => return,
        };
        storage.expire_keys();
    }
    pub fn generate_rdb(&self) -> Vec<u8> {
        let v: Vec<u8> = vec![0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31];
        v
    }
}
pub struct Serverinfo {
    pub host: String,
    pub port: u16,
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
        "echo" => {
            echo::command(server, &request, &command).await;
        }
        "ping" => {
            ping::command(server, &request, &command).await;
        }
        "set" => {
            set::command(server, &request, &command).await;
        }
        "get" => {
            get::command(server, &request, &command).await;
        }
        "info" => {
            info::command(server, &request, &command).await;
        }
        "replconf" => {
            replonf::command(&request, &command).await;
        }
        "psync" => {
            psync::command(&request, &command, server).await;
        }

        _ => {
            request
                .error(ServerError::CommandInternalError(command_name))
                .await;
        }
    }
}

pub async fn handshake(stream: &mut TcpStream, info: &Serverinfo) -> ServerResult {
    let mut buffer = [0u8; 512];

    let ping = RESP::Array(vec![RESP::BulkString("PING".to_string())]);
    let response = streams_send_receive_resp(stream, ping, &mut buffer)
        .await
        .map_err(|error| ServerError::HandshakeFailed(error.to_string()))?;
    if response != RESP::SimpleString("PONG".to_string()) {
        return Err(ServerError::HandshakeFailed(format!(
            "PING expected PONG, got {}",
            response
        )));
    }

    let listening_port = RESP::Array(vec![
        RESP::BulkString("REPLCONF".to_string()),
        RESP::BulkString("listening-port".to_string()),
        RESP::BulkString(info.port.to_string()),
    ]);
    let response = streams_send_receive_resp(stream, listening_port, &mut buffer)
        .await
        .map_err(|error| ServerError::HandshakeFailed(error.to_string()))?;
    if response != RESP::SimpleString("OK".to_string()) {
        return Err(ServerError::HandshakeFailed(format!(
            "REPLCONF listening-port expected OK, got {}",
            response
        )));
    }

    let capabilities = RESP::Array(vec![
        RESP::BulkString("REPLCONF".to_string()),
        RESP::BulkString("capa".to_string()),
        RESP::BulkString("psync2".to_string()),
    ]);
    let response = streams_send_receive_resp(stream, capabilities, &mut buffer)
        .await
        .map_err(|error| ServerError::HandshakeFailed(error.to_string()))?;
    if response != RESP::SimpleString("OK".to_string()) {
        return Err(ServerError::HandshakeFailed(format!(
            "REPLCONF capa expected OK, got {}",
            response
        )));
    }

    let psync = RESP::Array(vec![
        RESP::BulkString("PSYNC".to_string()),
        RESP::BulkString("?".to_string()),
        RESP::BulkString("-1".to_string()),
    ]);
    let response = streams_send_receive_resp(stream, psync, &mut buffer)
        .await
        .map_err(|error| ServerError::HandshakeFailed(error.to_string()))?;
    match response {
        RESP::SimpleString(value) if value.starts_with("FULLRESYNC ") => {}
        other => {
            return Err(ServerError::HandshakeFailed(format!(
                "PSYNC expected FULLRESYNC, got {}",
                other
            )));
        }
    }

    let rdb = stream_receive_resp(stream, &mut buffer)
        .await
        .map_err(|error| ServerError::HandshakeFailed(error.to_string()))?;
    if rdb != RESP::BulkString(String::new()) {
        return Err(ServerError::HandshakeFailed(format!(
            "PSYNC expected empty RDB payload, got {}",
            rdb
        )));
    }

    Ok(ServerValue::None)
}
