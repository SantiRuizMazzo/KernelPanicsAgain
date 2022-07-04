use crate::{
    client::client_side::ClientSide,
    config::Config,
    logger::torrent_logger::{Logger, Message},
    server::server_side::ServerSide,
    utils,
};
use std::env;

pub fn run() -> Result<(), String> {
    let logger: Logger = Logger::new("log.txt".to_string())?;
    let config = Config::new()?;
    let mut client = ClientSide::new(config)?;
    client.load_torrents(env::args())?;

    let mut server = ServerSide::new();
    server.set_peer_id(client.get_id());
    //server.init_server();
    client.init_client(logger.sender.clone())?;

    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );
    match logger.sender.send(Message::Log(log_peer_id)) {
        Err(error) => Err(error.to_string()),
        Ok(_) => Ok(()),
    }
}

pub fn run_ui() -> Result<(), String> {
    let logger: Logger = Logger::new("logtest.txt".to_string())?;
    let config = Config::new()?;
    let mut client = ClientSide::new(config)?;
    client.load_torrents(env::args())?;

    let mut server = ServerSide::new();
    server.set_peer_id(client.get_id());
    //server.init_server();
    client.init_client(logger.sender.clone())?;

    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );
    match logger.sender.send(Message::Log(log_peer_id)) {
        Err(error) => Err(error.to_string()),
        Ok(_) => Ok(()),
    }
}
