use crate::{
    client::client_side::ClientSide,
    logger::torrent_logger::{Logger, Message},
    server::server_side::ServerSide,
};
use std::env::args;

pub fn run() -> Result<(), String> {
    let logger: Logger = Logger::new("logtest.txt".to_string())?;
    // Comentar el init del server para probar peers reales.
    let mut client = ClientSide::new(8081);
    client.load_torrents(args())?;
    println!("{}", client.peer_id);
    let mut log_peer_id = "Client Peer ID:".to_string();
    log_peer_id += &client.peer_id;

    let mut server = ServerSide::new(8081);
    server.set_peer_id(client.peer_id.clone());
    server.init_server();
    client.init_client();

    match logger.sender.send(Message::Log(log_peer_id)) {
        Err(error) => Err(error.to_string()),
        Ok(_) => Ok(()),
    }
}
