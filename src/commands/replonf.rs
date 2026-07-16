use crate::request::Request;
use crate::resp::RESP;
use crate::server_result::ServerValue;

pub async fn command(request: &Request, command: &[String]) {
    let valid = match command {
        [name, option, value]
            if name.eq_ignore_ascii_case("replconf")
                && option.eq_ignore_ascii_case("listening-port") =>
        {
            match value.parse::<u16>() {
                Ok(port) if port > 0 => {
                    println!("REPLCONF listening-port {}: OK", port);
                    true
                }
                _ => false,
            }
        }
        [name, option, capability]
            if name.eq_ignore_ascii_case("replconf")
                && option.eq_ignore_ascii_case("capa")
                && capability.eq_ignore_ascii_case("psync2") =>
        {
            println!("REPLCONF capa {}: OK", capability);
            true
        }
        _ => false,
    };

    let response = if valid {
        RESP::SimpleString("OK".to_string())
    } else {
        RESP::Error("ERR invalid REPLCONF arguments".to_string())
    };

    request.data(ServerValue::RESP(response)).await;
}
