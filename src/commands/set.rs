use crate::request::Request;
use crate::resp::RESP;
use crate::server::Server;
use crate::server_result::{ServerError, ServerMessage, ServerValue};
use crate::set::parse_set_arguments;

pub async fn command(_server: &mut Server, _request: &Request,_command:&Vec< String>){
   let storage= match _server.storage.as_mut(){
      Some(storage)=>storage,
      None=>{
          _request.error(ServerError::StorageNotInititalised).await;
          return;
      }
   };
    if _command.len()<3{
        _request.error(ServerError::CommandSyntaxError(_command.join(" "))).await;
    }
    let key = _command[1].clone();
    let value = _command[2].clone();
    let args = match parse_set_arguments(&_command[3..].to_vec()){
        Ok(args)=>args,
        Err(error)=>{
            _request.error(ServerError::CommandSyntaxError(_command.join(" "))).await;
            return;
        }
    };
    if let Err(_) = storage.set(key, value, args){
        _request.error(ServerError::CommandInternalError(_command.join(" "))).await;
    }else{
        _request.data(ServerValue::RESP(RESP::SimpleString(String::from("OK")))).await;
    }
}  