use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::resp::{RESP,bytes_to_resp};
use crate::server::process_requeset;

mod resp;
mod resp_result;
mod server;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                tokio::spawn(async move {
                    handle_connection(&mut stream).await;
                });
            }
            Err(e) => {
                println!("Error accepting connection: {}", e);
                continue;
            }
        }
    }
}

async fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size != 0 => {
               let mut index :usize = 0;
               let request = match bytes_to_resp(&buffer[..size],&mut index){
                   Ok(resp)=>resp,
                   Err(e)=>{
                       eprintln!("Error parsing request: {}", e);
                       return;
                   }
               };
               let response = match process_requeset(request){
                   Ok(resp)=>resp,
                   Err(e)=>{
                       eprintln!("Error processing request: {}", e);
                       return;
                   }
               };
                if let Err(e) = stream.write_all(response.to_string().as_bytes()).await {
                    eprintln!("Error writing to stream: {}", e);
                    return;
                }
                if let Err(e) = stream.flush().await {
                    eprintln!("Error flushing stream: {}", e);
                    return;
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
}
