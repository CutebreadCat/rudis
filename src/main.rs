use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::resp::{RESP, bytes_to_resp};
use crate::server::process_requeset;
use crate::storage::Storage;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod resp;
mod resp_result;
mod server;
mod set;
mod storage;
mod storage_result;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let storage = Arc::new(Mutex::new(Storage::new()));
    let mut internaval_timer = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream, _)) => {
                        handle_connection(stream, storage.clone()).await;
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        continue;
                    }
                }
            }
            _ = internaval_timer.tick() => {
            tokio::spawn(expire_keys(storage.clone()));
        }

          }
    }
}

async fn handle_connection(mut stream: TcpStream, storage: Arc<Mutex<Storage>>) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size != 0 => {
                let mut index: usize = 0;
                let request = match bytes_to_resp(&buffer[..size], &mut index) {
                    Ok(resp) => resp,
                    Err(e) => {
                        eprintln!("Error parsing request: {}", e);
                        return;
                    }
                };
                let response = match process_requeset(request, storage.clone()) {
                    Ok(resp) => resp,
                    Err(e) => {
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

async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut guard = storage.lock().unwrap();
    guard.expire_keys();
    /*
    限制每次清理的数量：不要一次性遍历整个 expiry map，而是只处理固定数量（比如 20 个或 100 个），
    然后返回。这样即使 key 很多，也不会长时间阻塞。
    把清理任务放到阻塞线程池：用 tokio::task::spawn_blocking 运行真正的 CPU 密集型/阻塞型清理逻辑，
    避免占用异步调度线程。
    让清理过程可中断/可让出：如果确实要遍历很多 key，可以分段执行，并在段间 .await 一下 tokio::task::yield_now()，
    让其他任务有机会运行
     */
}
