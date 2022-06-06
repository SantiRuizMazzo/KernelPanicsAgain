use crate::messages::message_type::{
    choke::Choke, handshake::HandShake, have::Have, piece::Piece, unchoke::Unchoke,
};

use super::message_type::bitfield::Bitfield;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TorrentMessage {
    //Handshake(HandShake),
    Bitfield(Bitfield),
    Have(Have),
    Unchoke(Unchoke),
    //Interested(Interested),
    //NotInterested(NotInterested),
    //Request(Request),
    Piece(Piece),
    //Cancel(Cancel),
    Choke(Choke),
}
pub fn parse(bytes_read: Vec<u8>) -> Result<TorrentMessage, String> {
    match bytes_read[0] {
        0 => Ok(TorrentMessage::Choke(Choke::new())),
        1 => Ok(TorrentMessage::Unchoke(Unchoke::new())),
        //2 => Ok(TorrentMessage::Interested(Interested::new())),
        //3 => Ok(TorrentMessage::NotInterested(NotInterested::new())),
        4 => parse_have(bytes_read),
        5 => parse_bitfield(bytes_read),
        //6 => parse_request(bytes_read),
        7 => parse_piece(bytes_read),
        //8 => parse_cancel(bytes_read),
        _ => Err("Error while parsing message bytes read".to_string()),
    }
}
pub fn parse_piece(bytes: Vec<u8>) -> Result<TorrentMessage, String> {
    if !is_piece_message(bytes.clone()) {
        return Err("received message is not a bitfield".to_string());
    }
    let index = u32::from_be_bytes(
        bytes[1..5]
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
    );
    let begin = u32::from_be_bytes(
        bytes[5..9]
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
    );
    let block = Vec::from(&bytes[9..]);

    Ok(TorrentMessage::Piece(Piece::new(index, begin, block)))
}

pub fn parse_handshake(bytes: [u8; 68]) -> Result<HandShake, String> {
    if !is_handshake_message(bytes) {
        return Err("received message is not a handshake".to_string());
    }

    let protocol_length = bytes[0];
    let info_hash_index = (protocol_length + 9) as usize;
    let vec_bytes = bytes.to_vec();
    let info_hash_range = info_hash_index..(info_hash_index + 20);
    let info_hash = vec_bytes[info_hash_range].to_vec();
    let peer_id_range = (info_hash_index + 20)..vec_bytes.len();
    let peer_id = vec_bytes[peer_id_range].to_vec();

    Ok(HandShake::new(
        peer_id
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
        info_hash
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
    ))
}

pub fn parse_have(bytes: Vec<u8>) -> Result<TorrentMessage, String> {
    if !is_have_message(bytes.clone()) {
        return Err("received message is not a have".to_string());
    }
    let piece_index = u32::from_be_bytes(
        bytes[1..]
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
    );
    Ok(TorrentMessage::Have(Have::new(piece_index)))
}

pub fn parse_bitfield(bytes: Vec<u8>) -> Result<TorrentMessage, String> {
    if !is_bitfield_message(bytes.clone()) {
        return Err("received message is not a bitfield".to_string());
    }
    let bitfield = Vec::from(&bytes[1..]);
    Ok(TorrentMessage::Bitfield(Bitfield::new(
        bitfield.len() as u32,
        bitfield,
    )))
}

pub fn is_handshake_message(bytes: [u8; 68]) -> bool {
    let mut counter = 0;
    let mut bittorrent = [0; 19];
    let mut bittorrent_len = 0;
    for byte in bytes {
        if counter > 0 && bittorrent_len < 19 {
            bittorrent[bittorrent_len] = byte;
            counter += 1;
            bittorrent_len += 1;
        }
        counter += 1;
    }
    bittorrent == "BitTorrent protocol".as_bytes()
}

pub fn is_have_message(bytes: Vec<u8>) -> bool {
    bytes[0] == 4
}

pub fn is_bitfield_message(bytes: Vec<u8>) -> bool {
    bytes[0] == 5
}
pub fn is_piece_message(bytes: Vec<u8>) -> bool {
    bytes[0] == 7
}
/*
pub fn is_interested_message(bytes: [u8; 5]) -> bool {
    bytes[0] == 2
}
pub fn is_unchoke_message(bytes: [u8; 5]) -> bool {
    bytes[0] == 1
}
*/

/*pub fn is_request_message(bytes: [u8; 5]) -> bool {
    bytes[4] == 6
}*/
