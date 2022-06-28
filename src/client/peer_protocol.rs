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

pub fn download_piece(
    peer: Peer,
    client_id: [u8; 20],
    info_hash: [u8; 20],
    piece: TorrentPiece,
) -> Result<Vec<u8>, String> {
    let mut stream =
        TcpStream::connect(peer.get_connection_address()).map_err(|err| err.to_string())?;

    handle_handshake(&mut stream, peer, client_id, info_hash)?;
    let mut downloaded = Vec::<u8>::with_capacity(piece.get_length());
    //let mut bitfield;
    let mut cur_request = Request::new(piece.get_index() as u32, 0, 16384);

    loop {
        let len = read_len(&mut stream)?;
        if len == 0 {
            continue;
        }

        let bytes_read = read_id_and_payload(&mut stream, len)?;
        let message = message_parser::parse(bytes_read)?;
        println!("< RECEIVED: {:?}", message);

        match message {
            TorrentMessage::Bitfield(msg) => {
                /*bitfield = */
                handle_bitfield(&mut stream, msg.get_bits(), piece.get_index())?;
            }
            TorrentMessage::Have(_) => handle_have(&mut stream)?,
            TorrentMessage::Unchoke(_) => handle_unchoke(&mut stream, &mut cur_request)?,
            TorrentMessage::Piece(msg) => {
                let bytes_downloaded =
                    handle_piece(&mut stream, msg, &mut downloaded, piece, &mut cur_request)?;
                if bytes_downloaded == piece.get_length() {
                    break;
                }
            }
            _ => continue,
        }
    }
    Ok(downloaded)
}

fn handle_handshake(
    stream: &mut TcpStream,
    peer: Peer,
    client_id: [u8; 20],
    info_hash: [u8; 20],
) -> Result<(), String> {
    handle_hs_sending(stream, info_hash, client_id)?;
    handle_hs_receiving(stream, peer)?;
    Ok(())
}

fn handle_hs_sending(
    stream: &mut TcpStream,
    info_hash: [u8; 20],
    client_id: [u8; 20],
) -> Result<(), String> {
    let handshake = HandShake::new(
        "BitTorrent protocol".to_string(),
        [0u8; 8],
        info_hash,
        client_id,
    );
    handshake.send(stream).map_err(|err| err.to_string())?;
    println!("> SENT 🤝: {:?}", handshake);
    Ok(())
}

fn handle_hs_receiving(stream: &mut TcpStream, peer: Peer) -> Result<(), String> {
    let mut pstrlen = [0];
    stream
        .read_exact(&mut pstrlen)
        .map_err(|err| err.to_string())?;
    let pstrlen = u8::from_be_bytes(pstrlen) as usize;

    let mut bytes = vec![0; pstrlen + 48];
    stream
        .read_exact(&mut bytes)
        .map_err(|err| err.to_string())?;

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
    let peer_id = bytes[peer_id_idx..]
        .to_vec()
        .try_into()
        .map_err(|_| "conversion error".to_string())?;

    let handshake_response = HandShake::new(pstr, reserved, info_hash, peer_id);
    if handshake_response.has_same_peer_id(peer.get_id()) {
        println!("< RECEIVED 🤝: {:?}", handshake_response);
    }
    Ok(())
}

fn read_len(stream: &mut TcpStream) -> Result<u32, String> {
    let mut len = [0; 4];
    stream.read_exact(&mut len).map_err(|err| err.to_string())?;
    Ok(u32::from_be_bytes(len))
}

fn read_id_and_payload(stream: &mut TcpStream, len: u32) -> Result<Vec<u8>, String> {
    let mut bytes_read = vec![0; len as usize];
    stream
        .read_exact(&mut bytes_read)
        .map_err(|err| err.to_string())?;
    Ok(bytes_read)
}

fn handle_bitfield(
    stream: &mut TcpStream,
    bitfield: Vec<u8>,
    piece_index: usize,
) -> Result<Vec<u8>, String> {
    if !bitfield_contains(&bitfield, piece_index) {
        return Err("current remote peer does not serves this piece".to_string());
    }

    Interested::new()
        .send(stream)
        .map_err(|err| err.to_string())?;
    Ok(bitfield)
}

fn bitfield_contains(bitfield: &[u8], piece_index: usize) -> bool {
    let byte = bitfield[piece_index / 8];
    let mut shift = utils::round_up(piece_index, 8);
    if shift > 0 {
        shift -= 1;
    }
    let mask = 1 << shift;
    (byte & mask) != 0
}

fn handle_have(stream: &mut TcpStream) -> Result<(), String> {
    Interested::new()
        .send(stream)
        .map_err(|err| err.to_string())
}

fn handle_unchoke(stream: &mut TcpStream, cur_request: &mut Request) -> Result<(), String> {
    cur_request.send(stream).map_err(|err| err.to_string())
}

fn handle_piece(
    stream: &mut TcpStream,
    piece_msg: Piece,
    downloaded: &mut Vec<u8>,
    piece: TorrentPiece,
    cur_request: &mut Request,
) -> Result<usize, String> {
    if !(piece_msg.index == cur_request.index
        && piece_msg.begin == cur_request.begin
        && piece_msg.block.len() == cur_request.length as usize)
    {
        cur_request.send(stream).map_err(|err| err.to_string())?;
        return Ok(downloaded.len());
    }

    downloaded.append(&mut piece_msg.block.clone());
    let bytes_left = piece.get_length() - downloaded.len();

    if bytes_left == 0 {
        let expected_hash = piece.get_hash();
        let obtained_hash = utils::sha1(&downloaded)?;
        println!("EXPECTED HASH: {:x?}", expected_hash);
        println!("OBTAINED HASH: {:x?}", obtained_hash);

        if expected_hash != obtained_hash {
            println!("❌ NON-MATCHING HASHES ❌");
            return Err("downloaded bytes hash error".to_string());
        }
        println!("✅ MATCHING HASHES ✅");
        return Ok(downloaded.len());
    }

    let new_begin = piece_msg.begin + piece_msg.block.len() as u32;
    *cur_request = Request::new(
        piece.get_index() as u32,
        new_begin,
        cmp::min(16384, bytes_left as u32),
    );
    cur_request.send(stream).map_err(|err| err.to_string())?;
    Ok(downloaded.len())
}
