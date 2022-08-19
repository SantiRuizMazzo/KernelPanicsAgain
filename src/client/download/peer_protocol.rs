use super::download_pool::DownloadedPieces;
use crate::{
    client::piece::Piece,
    logging::log_handle::LogHandle,
    messages::message_types::{
        bitfield::Bitfield,
        block::Block,
        handshake::{Handshake, HANDSHAKE_PSTR},
        have::Have,
        interested::Interested,
        request::Request,
    },
    utils,
};
use std::{
    fmt::{self, Display, Formatter},
    io::Read,
    net::TcpStream,
};

pub const BLOCK_SIZE: u32 = 16384;

#[derive(Debug)]
pub enum ProtocolError {
    Peer(String),
    Piece(String),
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let display = match self {
            Self::Peer(e) => format!("Peer error: {e}"),
            Self::Piece(e) => format!("Piece error: {e}"),
        };
        write!(f, "{display}")
    }
}

pub fn handle_handshakes(
    stream: &mut TcpStream,
    peer_id: [u8; 20],
    info_hash: [u8; 20],
) -> Result<(), ProtocolError> {
    send_handshake(stream, peer_id, info_hash)?;
    receive_handshake(stream)?;
    Ok(())
}

pub fn send_handshake(
    stream: &mut TcpStream,
    peer_id: [u8; 20],
    info_hash: [u8; 20],
) -> Result<(), ProtocolError> {
    let handshake = Handshake::new(HANDSHAKE_PSTR, [0; 8], info_hash, peer_id);
    handshake.send(stream)
}

pub fn receive_handshake(stream: &mut TcpStream) -> Result<Handshake, ProtocolError> {
    let pstrlen = read_pstrlen_from(stream)?;
    let pstr = read_pstr_from(stream, pstrlen)?;
    let reserved = read_reserved_from(stream)?;
    let info_hash = read_info_hash_from(stream)?;
    let peer_id = read_peer_id_from(stream)?;
    Ok(Handshake::new(&pstr, reserved, info_hash, peer_id))
}

fn read_bytes_from(stream: &mut TcpStream, len: usize) -> Result<Vec<u8>, ProtocolError> {
    let mut bytes = vec![0; len];
    stream
        .read_exact(&mut bytes)
        .map_err(|e| ProtocolError::Peer(e.to_string()))?;
    Ok(bytes)
}

fn read_pstrlen_from(stream: &mut TcpStream) -> Result<u8, ProtocolError> {
    let pstrlen = read_bytes_from(stream, 1)?
        .try_into()
        .map_err(|_| ProtocolError::Peer("Conversion error for pstrlen field".to_string()))?;
    Ok(u8::from_be_bytes(pstrlen))
}

fn read_pstr_from(stream: &mut TcpStream, pstrlen: u8) -> Result<String, ProtocolError> {
    let pstr = read_bytes_from(stream, pstrlen as usize)?;
    utils::bytes_to_string(&pstr).map_err(ProtocolError::Peer)
}

fn read_reserved_from(stream: &mut TcpStream) -> Result<[u8; 8], ProtocolError> {
    read_bytes_from(stream, 8)?
        .try_into()
        .map_err(|_| ProtocolError::Peer("Conversion error for reserved field".to_string()))
}

fn read_info_hash_from(stream: &mut TcpStream) -> Result<[u8; 20], ProtocolError> {
    read_bytes_from(stream, 20)?
        .try_into()
        .map_err(|_| ProtocolError::Peer("Conversion error for info hash field".to_string()))
}

fn read_peer_id_from(stream: &mut TcpStream) -> Result<[u8; 20], ProtocolError> {
    read_bytes_from(stream, 20)?
        .try_into()
        .map_err(|_| ProtocolError::Peer("Conversion error for peer id field".to_string()))
}

fn read_len_from(stream: &mut TcpStream) -> Result<u32, ProtocolError> {
    let len = read_bytes_from(stream, 4)?
        .try_into()
        .map_err(|_| ProtocolError::Peer("Conversion error for len field".to_string()))?;
    Ok(u32::from_be_bytes(len))
}

pub fn read_message_bytes(stream: &mut TcpStream) -> Result<Vec<u8>, ProtocolError> {
    let len = read_len_from(stream)?;
    read_bytes_from(stream, len as usize)
}

pub fn handle_choke(last_request: &mut Request, am_choked: &mut bool) {
    last_request.reset();
    *am_choked = true
}

pub fn handle_unchoke(
    stream: &mut TcpStream,
    last_request: &mut Request,
    am_choked: &mut bool,
    am_interested: bool,
    _log_handle: LogHandle,
) -> Result<(), ProtocolError> {
    *am_choked = false;
    if am_interested {
        last_request.send(stream)?;
        //let _ = log_handle.log(&format!("> {last_request:?}"));
    }
    Ok(())
}

pub fn handle_have(
    stream: &mut TcpStream,
    have: Have,
    bitfield: &mut Bitfield,
    am_interested: &mut bool,
    downloaded_mutex: DownloadedPieces,
    _log_handle: LogHandle,
) -> Result<(), ProtocolError> {
    let new_piece_index = have.index() as usize;
    bitfield.add_piece(new_piece_index);

    let downloaded = downloaded_mutex
        .lock()
        .map_err(|e| ProtocolError::Peer(e.to_string()))?;

    let already_downloaded = downloaded
        .iter()
        .any(|piece| piece.index() == new_piece_index);

    if !already_downloaded && !*am_interested {
        *am_interested = true;
        let interested = Interested::new();
        interested.send(stream)?;
        //let _ = log_handle.log(&format!("> {interested:?}"));
    }
    Ok(())
}

pub fn handle_bitfield(
    stream: &mut TcpStream,
    bitfield: &mut Bitfield,
    piece_index: usize,
    am_interested: &mut bool,
    _log_handle: LogHandle,
) -> Result<(), ProtocolError> {
    if !bitfield.contains(piece_index) {
        let msg = format!("Remote peer is not serving piece {piece_index}");
        return Err(ProtocolError::Piece(msg));
    }

    *am_interested = true;
    let interested = Interested::new();
    interested.send(stream)?;
    //let _ = log_handle.log(&format!("> {interested:?}"));
    Ok(())
}

pub fn handle_request(
    stream: &mut TcpStream,
    request: Request,
    peer_is_choked: bool,
    download_path: String,
    bitfield: &Bitfield,
    _log_handle: LogHandle,
) -> Result<(), ProtocolError> {
    let err = ProtocolError::Peer;

    if peer_is_choked {
        let msg = "Remote peer is choked".to_string();
        return Err(err(msg));
    }

    if !bitfield.contains(request.index() as usize) {
        let cancel = request.cancel();
        cancel.send(stream)?;
        //log_handle.log(&format!("> {cancel:?}")).map_err(err)?;
        let msg = format!("Requested piece {} is not being served", request.index());
        return Err(err(msg));
    }

    let block_to_send = request.load_block_from(download_path).map_err(err)?;
    block_to_send.send(stream)?;
    //log_handle.log(&format!("> {block_to_send:?}")).map_err(err)
    Ok(())
}

pub fn handle_block(
    stream: &mut TcpStream,
    block: Block,
    piece: &mut Piece,
    last_request: &mut Request,
    am_choked: bool,
    _log_handle: LogHandle,
) -> Result<(), ProtocolError> {
    if am_choked {
        return Ok(());
    }

    if !last_request.matches(&block) {
        last_request.send(stream)?;
        //let _ = log_handle.log(&format!("> {last_request:?}"));
        return Ok(());
    }

    piece.append(&block);

    if piece.is_full() {
        if !piece.hashes_match() {
            let msg = format!("Hash verification for piece {} failed", piece.index());
            return Err(ProtocolError::Piece(msg));
        }
        return Ok(());
    }

    *last_request = piece.request_next_block();
    last_request.send(stream)?;
    //let _ = log_handle.log(&format!("> {last_request:?}"));
    Ok(())
}
