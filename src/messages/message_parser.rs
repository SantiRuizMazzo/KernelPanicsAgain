use crate::messages::message_type::{choke::Choke, have::Have, piece::Piece, unchoke::Unchoke};

use super::message_type::{
    bitfield::Bitfield, cancel::Cancel, interested::Interested, not_interested::NotInterested,
    request::Request,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PeerMessage {
    Bitfield(Bitfield),
    Have(Have),
    Unchoke(Unchoke),
    Piece(Piece),
    Choke(Choke),
    Interested(Interested),
    NotInterested(NotInterested),
    Request(Request),
    Cancel(Cancel),
}

pub fn parse(bytes_read: Vec<u8>) -> Result<PeerMessage, String> {
    match bytes_read[0] {
        0 => Ok(PeerMessage::Choke(Choke::new())),
        1 => Ok(PeerMessage::Unchoke(Unchoke::new())),
        2 => Ok(PeerMessage::Interested(Interested::new())),
        3 => Ok(PeerMessage::NotInterested(NotInterested::new())),
        4 => parse_have(bytes_read),
        5 => Ok(parse_bitfield(bytes_read)),
        6 => parse_request(bytes_read),
        7 => parse_piece(bytes_read),
        8 => parse_cancel(bytes_read),
        _ => Err("error while parsing message bytes read".to_string()),
    }
}

pub fn parse_cancel(bytes: Vec<u8>) -> Result<PeerMessage, String> {
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
    let length = u32::from_be_bytes(
        bytes[9..13]
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
    );

    Ok(PeerMessage::Cancel(Cancel::new(index, begin, length)))
}

pub fn parse_request(bytes: Vec<u8>) -> Result<PeerMessage, String> {
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
    let length = u32::from_be_bytes(
        bytes[9..13]
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
    );

    Ok(PeerMessage::Request(Request::new(
        index,
        begin,
        length as usize,
    )))
}

pub fn parse_piece(bytes: Vec<u8>) -> Result<PeerMessage, String> {
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

    Ok(PeerMessage::Piece(Piece::new(index, begin, block)))
}

pub fn parse_have(bytes: Vec<u8>) -> Result<PeerMessage, String> {
    if !is_have_message(bytes.clone()) {
        return Err("received message is not a have".to_string());
    }
    let piece_index = u32::from_be_bytes(
        bytes[1..]
            .try_into()
            .map_err(|_| "conversion error".to_string())?,
    );
    Ok(PeerMessage::Have(Have::new(piece_index)))
}

pub fn parse_bitfield(bytes: Vec<u8>) -> PeerMessage {
    let bitfield = Vec::from(&bytes[1..]);
    PeerMessage::Bitfield(Bitfield::new(bitfield))
}

pub fn is_have_message(bytes: Vec<u8>) -> bool {
    bytes[0] == 4
}
