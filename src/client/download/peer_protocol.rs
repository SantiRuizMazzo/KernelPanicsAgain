use super::download_info::DownloadInfo;
use crate::{
    client::torrent_piece::TorrentPiece,
    messages::message_type::{
        handshake::HandShake, have::Have, interested::Interested, piece::Piece, request::Request,
    },
    utils,
};

use std::{cmp, io::Read, net::TcpStream};

pub const BLOCK_SIZE: usize = 16384;

pub enum DownloadError {
    Connection(String),
    Piece(String),
}

pub fn handle_handshake(
    stream: &mut TcpStream,
    peer_id: Option<[u8; 20]>,
    download: DownloadInfo,
) -> Result<(), String> {
    handle_hs_sending(stream, download)?;
    handle_hs_receiving(stream, peer_id)?;
    Ok(())
}

fn handle_hs_sending(stream: &mut TcpStream, download: DownloadInfo) -> Result<(), String> {
    let handshake = HandShake::new(
        "BitTorrent protocol".to_string(),
        [0u8; 8],
        download.get_hash(),
        download.get_id(),
    );
    handshake.send(stream).map_err(|err| err.to_string())?;
    Ok(())
}

fn handle_hs_receiving(stream: &mut TcpStream, peer_id: Option<[u8; 20]>) -> Result<(), String> {
    let mut pstrlen = [0];
    let mut read = stream.read(&mut pstrlen).map_err(|err| err.to_string())?;
    if read != 1 {
        return Err("not enough bytes to read pstrlen field".to_string());
    }
    let pstrlen = u8::from_be_bytes(pstrlen) as usize;
    let handshake_len = pstrlen + 48;

    let mut bytes = vec![0; handshake_len];
    read = stream.read(&mut bytes).map_err(|err| err.to_string())?;
    if read != handshake_len {
        return Err("not enough bytes to read handshake fields".to_string());
    }

    let pstr = utils::bytes_to_string(&bytes[..pstrlen].to_vec())?;
    let info_hash_idx = pstrlen + 8;
    let peer_id_idx = info_hash_idx + 20;
    let reserved = bytes[pstrlen..info_hash_idx]
        .to_vec()
        .try_into()
        .map_err(|_| "conversion error".to_string())?;
    let info_hash = bytes[info_hash_idx..peer_id_idx]
        .to_vec()
        .try_into()
        .map_err(|_| "conversion error".to_string())?;
    let read_peer_id = bytes[peer_id_idx..]
        .to_vec()
        .try_into()
        .map_err(|_| "conversion error".to_string())?;

    let handshake_response = HandShake::new(pstr, reserved, info_hash, read_peer_id);
    if handshake_response.has_same_peer_id(peer_id) {}
    Ok(())
}

pub fn read_len(stream: &mut TcpStream) -> Result<u32, DownloadError> {
    let mut len = [0; 4];
    let read = stream
        .read(&mut len)
        .map_err(|err| DownloadError::Connection(err.to_string()))?;
    if read == 0 {
        return Err(DownloadError::Connection(
            "connection closed by remote peer".to_string(),
        ));
    }
    Ok(u32::from_be_bytes(len))
}

pub fn read_id_and_payload(stream: &mut TcpStream, len: u32) -> Result<Vec<u8>, DownloadError> {
    let mut bytes_read = vec![0; len as usize];
    stream
        .read_exact(&mut bytes_read)
        .map_err(|err| DownloadError::Connection(err.to_string()))?;
    Ok(bytes_read)
}

pub fn handle_bitfield(
    stream: &mut TcpStream,
    bitfield: Vec<u8>,
    piece_index: usize,
    am_interested: &mut bool,
) -> Result<Vec<u8>, DownloadError> {
    if !bitfield_contains(&bitfield, piece_index) {
        return Err(DownloadError::Piece(
            "current remote peer does not serves this piece".to_string(),
        ));
    }

    *am_interested = true;
    Interested::new()
        .send(stream)
        .map_err(|err| DownloadError::Piece(err.to_string()))?;
    Ok(bitfield)
}

pub fn bitfield_contains(bitfield: &[u8], piece_index: usize) -> bool {
    let byte = bitfield[piece_index / 8];
    let mask = piece_bit_mask(piece_index);
    (byte & mask) != 0
}

pub fn handle_have(
    stream: &mut TcpStream,
    have_msg: Have,
    bitfield: &mut Vec<u8>,
    am_interested: &mut bool,
    piece_index: usize,
) -> Result<(), DownloadError> {
    let new_piece_index = have_msg.get_index() as usize;
    bitfield[new_piece_index / 8] |= piece_bit_mask(new_piece_index);

    if !*am_interested && bitfield_contains(bitfield, piece_index) {
        *am_interested = true;
        Interested::new()
            .send(stream)
            .map_err(|err| DownloadError::Piece(err.to_string()))?;
    }
    Ok(())
}

pub fn piece_bit_mask(piece_index: usize) -> u8 {
    let byte_end = utils::round_up(piece_index, 8);
    let mut shift = 7;
    if byte_end > piece_index {
        shift = byte_end - 1 - piece_index;
    }
    1 << shift
}

pub fn handle_unchoke(
    stream: &mut TcpStream,
    cur_request: &mut Request,
    am_choked: &mut bool,
    am_interested: bool,
) -> Result<(), DownloadError> {
    *am_choked = false;
    if am_interested {
        cur_request
            .send(stream)
            .map_err(|err| DownloadError::Piece(err.to_string()))?;
    }
    Ok(())
}

pub fn handle_piece(
    stream: &mut TcpStream,
    piece_msg: Piece,
    downloaded: &mut Vec<u8>,
    piece: TorrentPiece,
    cur_request: &mut Request,
    am_choked: bool,
) -> Result<usize, DownloadError> {
    if am_choked {
        return Ok(downloaded.len());
    }

    if !(piece_msg.index == cur_request.index
        && piece_msg.begin == cur_request.begin
        && piece_msg.block.len() == cur_request.length as usize)
    {
        cur_request
            .send(stream)
            .map_err(|err| DownloadError::Piece(err.to_string()))?;
        return Ok(downloaded.len());
    }

    downloaded.append(&mut piece_msg.block.clone());
    let bytes_left = piece.get_length() - downloaded.len();

    if bytes_left == 0 {
        let expected_hash = piece.get_hash();
        let obtained_hash = utils::sha1(&downloaded).map_err(DownloadError::Piece)?;

        if expected_hash != obtained_hash {
            return Err(DownloadError::Piece(
                "downloaded bytes hash error".to_string(),
            ));
        }
        return Ok(downloaded.len());
    }

    let new_begin = piece_msg.begin + piece_msg.block.len() as u32;
    *cur_request = Request::new(
        piece.get_index() as u32,
        new_begin,
        cmp::min(BLOCK_SIZE, bytes_left),
    );
    cur_request
        .send(stream)
        .map_err(|err| DownloadError::Piece(err.to_string()))?;
    Ok(downloaded.len())
}

pub fn handle_choke(cur_request: &mut Request, am_choked: &mut bool) {
    *am_choked = true;
    cur_request.discarded()
}
