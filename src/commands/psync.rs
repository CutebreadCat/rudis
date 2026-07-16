use crate::request::Request;
use crate::resp::RESP;
use crate::server::Server;
use crate::server_result::{ServerError, ServerValue};

pub async fn command(request: &Request, command: &[String], server: &Server) {
    if command.len() != 3 {
        request
            .error(ServerError::CommandSyntaxError(command.join(" ")))
            .await;
        return;
    }

    let requested_replid = &command[1];
    let requested_offset = &command[2];

    println!(
        "PSYNC requested replid={} offset={}",
        requested_replid, requested_offset
    );

    let response = format!(
        "FULLRESYNC {} {}",
        server.replication.reolid, server.replication.repl_offset
    );

    println!("{}", response);
    request
        .data(ServerValue::RESP(RESP::SimpleString(response)))
        .await;

    let rdb_payload: Vec<u8> = Vec::new();
    let mut rdb = format!("${}\r\n", rdb_payload.len()).into_bytes();
    rdb.extend_from_slice(&rdb_payload);
    rdb.extend_from_slice(b"\r\n");

    request.data(ServerValue::Binary(rdb)).await;
}
