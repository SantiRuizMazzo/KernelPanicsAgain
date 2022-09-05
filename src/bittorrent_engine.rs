use crate::{
    client::client_side::ClientSide, config::Config, logging::logger::Logger,
    server::server_side::ServerSide,
};
use std::sync::mpsc;

pub fn run() -> Result<(), String> {
    let config = Config::new()?;
    let logger = Logger::new(config.log_path())?;

    let mut client = ClientSide::new(&config, logger.handle());
    let mut server = ServerSide::new(client.get_id(), &config, logger.handle());

    let (notif_tx, notif_rx) = mpsc::channel();
    server.init(notif_tx.clone(), notif_rx)?;
    let mut download_pool = client.init(notif_tx)?;
    download_pool.wait_for_workers();
    Ok(())
}
