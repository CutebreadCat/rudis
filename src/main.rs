use crate::request::Request;
use crate::resp::{RESP, bytes_to_resp};
use crate::server::process_requeset;
use crate::storage::Storage;
use server::{Server, run_server};
use server_result::ServerMessage;
use server_result::ServerValue;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};

use crate::connection::{ConnectionError, ConnectionMessage, run_listner};

mod connection;
mod request;
mod resp;
mod resp_result;
mod server;
mod server_result;
mod set;
mod storage;
mod storage_result;

#[tokio::main]
async fn main() {
    let storage = Storage::new();
    let mut server = Server::new(storage);
    let (server_sender, server_receiver) = mpsc::channel::<ConnectionMessage>(32);
    tokio::spawn({ run_server(server, server_receiver) });
    run_listner("127.0.0.1".to_string(), 6379, server_sender).await;
}
