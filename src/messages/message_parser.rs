use crate::messages::message_type::{choke::Choke, have::Have, piece::Piece, unchoke::Unchoke};

use super::message_type::bitfield::Bitfield;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TorrentMessage {
    Bitfield(Bitfield),
    Have(Have),
    Unchoke(Unchoke),
    Piece(Piece),
    Choke(Choke),
}
pub fn parse(bytes_read: Vec<u8>) -> Result<TorrentMessage, String> {
    match bytes_read[0] {
        0 => Ok(TorrentMessage::Choke(Choke::new())),
        1 => Ok(TorrentMessage::Unchoke(Unchoke::new())),
        4 => parse_have(bytes_read),
        5 => parse_bitfield(bytes_read),
        7 => parse_piece(bytes_read),
        _ => Err("error while parsing message bytes read".to_string()),
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

pub fn is_have_message(bytes: Vec<u8>) -> bool {
    bytes[0] == 4
}

pub fn is_bitfield_message(bytes: Vec<u8>) -> bool {
    bytes[0] == 5
}

pub fn is_piece_message(bytes: Vec<u8>) -> bool {
    bytes[0] == 7
}
