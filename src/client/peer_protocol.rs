use super::peer::Peer;
use crate::messages::{message_parser, message_type::handshake::HandShake};
use std::{io::Read, net::TcpStream};

pub fn handle_communication(
    remote_peer: Peer,
    client_id: [u8; 20],
    info_hash: [u8; 20],
) -> Result<(), String> {
    let mut stream =
        TcpStream::connect(remote_peer.get_connection_address()).map_err(|err| err.to_string())?;

    //ESCRITURA DEL HANDSHAKE
    let handshake = HandShake::new(client_id, info_hash);
    println!("> PARSED HANDSHAKE SENT: {:?}", handshake);
    handshake.send(&mut stream).map_err(|err| err.to_string())?;

    //LECTURA DEL HANDSHAKE
    let mut handshake_bytes = [0; 68];
    stream
        .read_exact(&mut handshake_bytes)
        .map_err(|err| err.to_string())?;
    println!("> RAW HANDSHAKE RECEIVED: {:x?}", handshake_bytes);

    //PARSEO DEL HANDSHAKE
    let handshake_response = message_parser::parse_handshake(handshake_bytes)?;
    if handshake_response.has_same_peer_id(remote_peer.get_id()) {
        println!("> PARSED HANDSHAKE RECEIVED 🤝: {:?}", handshake_response);
    }

    //LECTURA DEL HAVE
    let mut have_bytes = [0; 6];
    stream
        .read_exact(&mut have_bytes)
        .map_err(|err| err.to_string())?;
    println!("> RAW HAVE RECEIVED: {:?}", have_bytes);

    //PARSEO DEL HAVE
    let parsed_have = message_parser::parse_have(have_bytes)?;
    println!("> PARSED HAVE RECEIVED: {:?}", parsed_have);
    Ok(())
}
