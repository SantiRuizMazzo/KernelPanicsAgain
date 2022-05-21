use crate::{
    client::client_side::ClientSide,
    logger::torrent_logger::{Logger, Message},
};
pub fn run() -> Result<(), String> {
    let logger: Logger = Logger::new("logtest.txt".to_string())?;
    let client = ClientSide::new();
    println!("{}", client.peer_id);
    let mut log_peer_id = "Client Peer ID:".to_string();
    log_peer_id += &client.peer_id;
    match logger.sender.send(Message::Log(log_peer_id)) {
        Err(error) => Err(error.to_string()),
        Ok(_) => Ok(()),
    }
}
