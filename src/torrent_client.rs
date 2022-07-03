use crate::{
    client::client_side::ClientSide,
    logger::torrent_logger::{Logger, Message},
    server::server_side::ServerSide,
    utils,
};
use std::env;

pub fn run() -> Result<(), String> {
    let logger: Logger = Logger::new("logtest.txt".to_string())?;
    let mut client = ClientSide::new(8081)?;
    client.load_torrents(env::args())?;

    let mut server = ServerSide::new(8081);
    server.set_peer_id(client.get_id());
    //server.init_server();
    client.init_client()?;

    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );
    match logger.sender.send(Message::Log(log_peer_id)) {
        Err(error) => Err(error.to_string()),
        Ok(_) => Ok(()),
    }
}
