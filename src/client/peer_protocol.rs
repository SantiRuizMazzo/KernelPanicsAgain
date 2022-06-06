use super::{peer::Peer, torrent_piece::TorrentPiece};
use crate::{
    messages::{
        message_parser::{self, TorrentMessage},
        message_type::{
            handshake::HandShake, interested::Interested, piece::Piece, request::Request,
        },
    },
    utils,
};
use std::{cmp, io::Read, net::TcpStream};

pub fn handle_communication(
    remote_peer: Peer,
    client_id: [u8; 20],
    info_hash: [u8; 20],
    target_piece: TorrentPiece,
) -> Result<Vec<u8>, String> {
    let mut stream =
        TcpStream::connect(remote_peer.get_connection_address()).map_err(|err| err.to_string())?;

    handle_handshake(&mut stream, remote_peer, client_id, info_hash)?;
    let mut downloaded_bytes = Vec::<u8>::with_capacity(target_piece.get_length());

    loop {
        let len = read_len(&mut stream)?;
        println!("> READ MESSAGE LEN: {}", len);

        if len == 0 {
            println!("> RECEIVED KEEP ALIVE...");
            continue;
        }

        let bytes_read = read_id_and_payload(&mut stream, len)?;
        println!("> READ RAW BYTES (ID + PAYLOAD): {:?}", bytes_read);

        let message = message_parser::parse(bytes_read)?;
        println!("> PARSED MESSAGE: {:?}", message);

        match message {
            TorrentMessage::Bitfield(_) => handle_bitfield(&mut stream)?,
            TorrentMessage::Have(_) => handle_have(&mut stream)?,
            TorrentMessage::Unchoke(_) => handle_unchoke(&mut stream, target_piece)?,
            TorrentMessage::Piece(piece) => {
                match handle_piece(&mut stream, piece, &mut downloaded_bytes, target_piece) {
                    Ok(bytes_length) => {
                        if bytes_length == target_piece.get_length() {
                            break;
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
            _ => continue,
        }
    }
    Ok(downloaded_bytes)
}

fn handle_handshake(
    stream: &mut TcpStream,
    remote_peer: Peer,
    client_id: [u8; 20],
    info_hash: [u8; 20],
) -> Result<(), String> {
    let handshake = HandShake::new(client_id, info_hash);
    println!("> PARSED HANDSHAKE SENT: {:?}", handshake);
    handshake.send(stream).map_err(|err| err.to_string())?;

    let mut handshake_bytes = [0; 68];
    stream
        .read_exact(&mut handshake_bytes)
        .map_err(|err| err.to_string())?;

    let handshake_response = message_parser::parse_handshake(handshake_bytes)?;
    if handshake_response.has_same_peer_id(remote_peer.get_id()) {
        println!("> PARSED HANDSHAKE RECEIVED 🤝: {:?}", handshake_response);
    }
    Ok(())
}

fn read_len(stream: &mut TcpStream) -> Result<u32, String> {
    let mut len = [0; 4];
    stream.read_exact(&mut len).map_err(|err| err.to_string())?;
    Ok(u32::from_be_bytes(len))
}

fn read_id_and_payload(stream: &mut TcpStream, len: u32) -> Result<Vec<u8>, String> {
    let mut bytes_read = vec![0_u8; len as usize];
    stream
        .read_exact(&mut bytes_read)
        .map_err(|err| err.to_string())?;
    Ok(bytes_read)
}

fn handle_bitfield(stream: &mut TcpStream) -> Result<(), String> {
    Interested::new()
        .send(stream)
        .map_err(|err| err.to_string())
}

fn handle_have(stream: &mut TcpStream) -> Result<(), String> {
    Interested::new()
        .send(stream)
        .map_err(|err| err.to_string())
}

fn handle_unchoke(stream: &mut TcpStream, target_piece: TorrentPiece) -> Result<(), String> {
    Request::new(target_piece.get_index() as u32, 0, 16384)
        .send(stream)
        .map_err(|err| err.to_string())
}

fn handle_piece(
    stream: &mut TcpStream,
    piece_msg: Piece,
    downloaded_bytes: &mut Vec<u8>,
    target_piece: TorrentPiece,
) -> Result<usize, String> {
    downloaded_bytes.append(&mut piece_msg.block.clone());
    let bytes_left = target_piece.get_length() - downloaded_bytes.len();

    if bytes_left == 0 {
        if target_piece.get_hash() == utils::sha1(downloaded_bytes.clone())? {
            println!("✅ DOWNLOADED BYTES HASH IS CORRECT ✅");
            return Ok(downloaded_bytes.len());
        } else {
            println!("❌ DOWNLOADED BYTES HASH IS INCORRECT ❌");
            return Err("downloaded bytes hash error".to_string());
        }
    }

    let new_begin = piece_msg.begin + piece_msg.block.len() as u32;
    Request::new(
        target_piece.get_index() as u32,
        new_begin,
        cmp::min(16384, bytes_left as u32),
    )
    .send(stream)
    .map_err(|err| err.to_string())?;
    Ok(downloaded_bytes.len())
}
