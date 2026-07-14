use crate::request::Request;
use crate::resp::RESP;
use crate::server::Server;
use crate::server_result::ServerValue;

pub async fn command(_server: &mut Server, _request: &Request,_command:&Vec< String>){
    _request.data(ServerValue::RESP(RESP::SimpleString(_command[1].to_string()))).await;
}