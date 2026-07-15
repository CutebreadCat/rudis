use std::process::Output;

use crate::replication::Role;
use crate::request::Request;
use crate::resp::bulk_string_from_vec;
use crate::server::Server;
use crate::server_result::ServerValue;
use crate::storage::StorageValue;

pub async fn command(server: &Server, request: &Request, _command: &Vec<String>) {
    let replication_info = server.replication.info();
    let mut output = vec![String::from("# Replication")];

    match replication_info.role {
        Role::Master => {
            output.push(format!("role:master"));
        }
        Role::Replica => {
            output.push(format!("role:slave"));
        }
    }
    output.push(format!("master_replid:{}", replication_info.reolid));
    output.push(format!(
        "master_repl_offset:{}",
        replication_info.repl_offset
    ));
    request
        .data(ServerValue::RESP(bulk_string_from_vec(output)))
        .await;
}
