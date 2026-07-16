use crate::replication::Role::Master;
use crate::resp::RESP;

use crate::storage::Storage;
use clap::Parser;
use server::{Server, run_server};
use tokio::sync::mpsc;

use crate::connection::{ConnectionMessage, run_listner, run_master_listener};
use crate::replication::ReplicationConfig;

mod commands;
mod connection;
mod replication;
mod request;
mod resp;
mod resp_result;
mod server;
mod server_result;
mod set;
mod storage;
mod storage_result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
    short = 'H',
    long,
    help = "The host address to bind the server to",
    default_value_t = String::from("127.0.0.1")
    )]
    host: String,

    #[arg(
        short = 'P',
        long,
        help = "The port to bind the server to",
        default_value_t = 6379
    )]
    port: u16,

    #[arg(
        short,
        long,
        help = "The matset server for this replica,in the form `address port`"
    )]
    replicaof: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    
    let args = Args::parse();
    let mut server = Server::new(args.host.clone(), args.port);
    let replication_config = match args.replicaof {
        None => ReplicationConfig::new_master(),
        Some(params) => {
            let (host, port_string) = match params.split_once(' ') {
                Some(value) => (value),
                None => {
                    eprintln!("Invalid replicaof parameter: {}", params);
                    std::process::exit(1);
                }
            };
            let port: u16 = port_string.parse().unwrap_or(6379);
            ReplicationConfig::new_replica(host.to_owned(), port)
        }
    };

    let storage = Storage::new();
    let mut server = Server::new(args.host.clone(), args.port);
    server.set_replication(replication_config);
    let (server_sender, server_receiver) = mpsc::channel::<ConnectionMessage>(32);
    if let Some(master_config) = server.replication.master.clone() {
        run_master_listener(
            master_config.host.clone(),
            master_config.port,
            server_sender.clone(),
            &server.info,
        )
        .await;
    }
    tokio::spawn(run_server(server, server_receiver));
    run_listner(args.host, args.port, server_sender).await;
    Ok(())
}
