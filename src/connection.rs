use crate::request::{self, Request};
use crate::resp::{RESP, bytes_to_resp};
use crate::server::{Serverinfo, handshake};
use crate::server_result::ServerResult;
use crate::server_result::{ServerError, ServerMessage, ServerValue};
use std::fmt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};

#[derive(Debug)]
pub enum ConnectionError {
    ServerError(ServerError),
    CannotWriteToStream(String),
    CannotReadFromStream(String),
    MalformedRESP(String),
    ReuquestError(String, String),
}
impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionError::ServerError(e) => write!(f, "{}", e),
            ConnectionError::CannotWriteToStream(e) => write!(f, "Cannot write to stream: {}", e),
            ConnectionError::CannotReadFromStream(e) => write!(f, "Cannot read from stream: {}", e),
            ConnectionError::MalformedRESP(e) => write!(f, "Malformed RESP: {}", e),
            ConnectionError::ReuquestError(request, e) => {
                write!(f, "ReuquestError: {} {}", request, e)
            }
        }
    }
}
type ConnectionResult<T> = std::result::Result<T, ConnectionError>;
#[derive(Debug)]
pub enum ConnectionMessage {
    Request(Request),
}

async fn handle_connection(mut stream: TcpStream, server_send: mpsc::Sender<ConnectionMessage>) {
    let mut buffer = [0; 512];
    let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);
    loop {
        select! {
            result = stream.read(&mut buffer)=> {

            match result{
                Ok(size) if size != 0 => {
                    let mut index: usize = 0;
                    let resp = match bytes_to_resp(&buffer[..size], &mut index) {
                        Ok(resp) => resp,
                        Err(e) => {
                            eprintln!("Error parsing request: {}", e);
                            return;
                        }
                    };
                        let request = Request{
                            resp: resp,
                            sender: connection_sender.clone(),
                        };
                    match server_send.send(ConnectionMessage::Request(request)).await{
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error sending request to server: {}", e);
                                return;
                            }
                        }
                   }
                Ok(_) => {
                    println!("Connection closed by client");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
            Some(reponse) = connection_receiver.recv()=>{
                let _ =match reponse{

                    ServerMessage::Data(ServerValue::RESP(v))=>stream.write_all(v.to_string().as_bytes()).await,
                    ServerMessage::Data(ServerValue::Binary(bytes))=>stream.write_all(&bytes).await,
                    ServerMessage::Data(ServerValue::None)=>{Ok(())}
                    ServerMessage::Error(v)=>{
                        eprintln!("Error: {}", v);
                        return;

                },
                };


            }
            }
    }
}
pub async fn run_listner(host: String, port: u16, server_sender: mpsc::Sender<ConnectionMessage>) {
    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream, _)) => {
                        tokio::spawn(handle_connection(stream, server_sender.clone()));
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        continue;
                    }
                }
            }

        }
    }
}

pub async fn run_master_listener(
    host: String,
    port: u16,
    server_sender: mpsc::Sender<ConnectionMessage>,
    info: &Serverinfo,
) {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))
        .await
        .unwrap();
    if let Err(e) = handshake(&mut stream, info).await {
        eprintln!("Handshake failed :{}", e.to_string());
        return;
    }
    tokio::spawn(async move { handle_connection(stream, server_sender.clone()).await });
}

pub async fn stream_write(stream: &mut TcpStream, data: RESP) -> ConnectionResult<usize> {
    let data = data.to_string();
    let len = data.len();
    match stream.write_all(data.as_bytes()).await {
        Ok(_) => Ok(len),
        Err(e) => Err(ConnectionError::CannotWriteToStream(e.to_string())),
    }
}
pub async fn stream_receive_resp(
    stream: &mut TcpStream,
    buffer: &mut [u8],
) -> ConnectionResult<RESP> {
    let size = stream
        .read(buffer)
        .await
        .map_err(|error| ConnectionError::CannotReadFromStream(error.to_string()))?;
    if size == 0 {
        return Err(ConnectionError::CannotReadFromStream(
            "connection closed by peer".to_string(),
        ));
    }

    let mut index = 0;
    bytes_to_resp(&buffer[..size], &mut index)
        .map_err(|error| ConnectionError::MalformedRESP(error.to_string()))
}

pub async fn streams_send_receive_resp(
    stream: &mut TcpStream,
    data: RESP,
    buffer: &mut [u8],
) -> ConnectionResult<RESP> {
    stream_write(stream, data)
        .await
        .map_err(|e| ConnectionError::ServerError(ServerError::HandshakeFailed(e.to_string())))?;
    stream_receive_resp(stream, buffer)
        .await
        .map_err(|e| ConnectionError::ServerError(ServerError::HandshakeFailed(e.to_string())))
}
