use std::env;

use crate::{
    client::client_side::ClientSide,
    config::Config,
    logger::torrent_logger::{LogMessage, Logger},
    server::server_side::ServerSide,
    utils,
};

pub fn run() -> Result<(), String> {
    let config = Config::new()?;
    let logger = Logger::new(config.clone())?;

    let mut client = ClientSide::new(config)?;
    let mut server = ServerSide::new();
    server.set_peer_id(client.get_id());
    //server.init();
    client.load_torrents(env::args())?;
    client.init(logger.get_sender())?;

    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );

    logger
        .get_sender()
        .send(LogMessage::Log(log_peer_id))
        .map_err(|err| err.to_string())?;
    Ok(())
}
