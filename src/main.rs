use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},

};

mod resp;
mod resp_result;


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


async  fn handle_connection(stream : &mut TcpStream){
   
    let mut buffer = [0; 512];
   loop{
    match stream.read(&mut buffer).await{
        Ok(size)if size != 0 => {
          
            let response = "+PONG\r\n";
            if let Err(e) = stream.write_all(response.as_bytes()).await {
                eprintln!("Error writing to stream: {}", e);
            }
        }
        Ok(_)=>{
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