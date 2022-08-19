use crate::{
    client::client_side::ClientSide, config::Config, logging::logger::Logger,
    server::server_side::ServerSide, utils,
};
use std::sync::mpsc;

pub fn run() -> Result<(), String> {
    let config = Config::new()?;
    let logger = Logger::new(config.log_path())?;

    let mut client = ClientSide::new(&config, logger.handle());
    let mut server = ServerSide::new(client.get_id(), &config, logger.handle());

    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );

    logger.handle().log(&log_peer_id)?;

    let (notif_tx, notif_rx) = mpsc::channel();
    server.init(notif_tx.clone(), notif_rx)?;
    client.init(notif_tx)
}
