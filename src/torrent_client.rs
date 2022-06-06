use crate::{
    client::client_side::ClientSide,
    logger::torrent_logger::{Logger, Message},
    server::server_side::ServerSide,
    utils,
};
use std::env::args;

pub fn run() -> Result<(), String> {
    let logger: Logger = Logger::new("logtest.txt".to_string())?;
    let mut client = ClientSide::new(8081)?;
    client.load_torrents(args())?;
    println!("{:?}", client.get_id());

    let mut server = ServerSide::new(8081);
    server.set_peer_id(client.get_id());
    // Comentar el init del server para probar peers reales.
    //server.init_server();
    println!("{}", client.init_client()?);

    let log_peer_id = format!(
        "Client Peer ID: {}",
        utils::bytes_to_string(&client.get_id())?
    );
    match logger.sender.send(Message::Log(log_peer_id)) {
        Err(error) => Err(error.to_string()),
        Ok(_) => Ok(()),
    }
}
