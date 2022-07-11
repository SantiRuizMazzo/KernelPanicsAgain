use std::{env, sync::mpsc};

use crate::{
    client::client_side::ClientSide,
    config::Config,
    logger::torrent_logger::{LogMessage, Logger},
    server::server_side::ServerSide,
    utils,
};

pub fn run() -> Result<(), String> {
    let (notif_tx, notif_rx) = mpsc::channel();
    let config = Config::new()?;
    let logger = Logger::new(config.clone())?;

    let mut client = ClientSide::new(config.clone(), notif_tx.clone())?;
    let mut server = ServerSide::new(config, logger.get_sender());

    server.set_peer_id(client.get_id());
    server.init(notif_tx, notif_rx)?;
    client.init(logger.get_sender())?;
    client.load_torrents(env::args())?;

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
