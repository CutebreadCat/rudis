use crate::request::Request;
use crate::resp::RESP;
use crate::server::Server;
use crate::server_result::{ServerError, ServerValue};

pub async fn command(_server: &mut Server, _request: &Request,_command:&Vec< String>){
   let storage = match _server.storage.as_mut(){
      Some(storage) => storage,
      None => {
          _request.error(ServerError::StorageNotInititalised).await;
          return;
      }
   };
    if _command.len() < 2 {
        _request.error(ServerError::CommandSyntaxError(_command.join(" "))).await;
        return;
    }
    let output = match storage.get(_command[1].clone()){
     Ok(Some(v))=>_request.data(ServerValue::RESP(RESP::BulkString( v))).await,
        Ok(None)=>_request.data(ServerValue::RESP(RESP::Null)).await,
        Err(e)=>_request.error(ServerError::CommandSyntaxError(_command.join(" "))).await,
    };
}