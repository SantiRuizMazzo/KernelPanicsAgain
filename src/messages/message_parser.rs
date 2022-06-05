//use super::message_type::bitfield::Bitfield;
use crate::messages::message_type::handshake::HandShake;
use crate::messages::message_type::have::Have;

pub fn parse_handshake(bytes: [u8; 69]) -> HandShake<'static> {
    /*
    let mut counter = 0;
    let mut info_hash_len = 0;
    let mut info_hash = [0; 20];
    let mut peer_id = [0; 20];
    let mut peer_id_len = 0;
    for byte in bytes {
        if byte == 0 && counter != 8 {
            counter += 1;
        } else if counter == 8 && info_hash_len < 20 {
            info_hash[info_hash_len] = byte;
            info_hash_len += 1;
        } else if info_hash_len >= 20 && byte != 10 {
            peer_id[peer_id_len] = byte;
            peer_id_len += 1;
        } else {
            counter = 0
        }
    }
    */
    let protocol_length = bytes[0];
    let info_hash_index = (protocol_length + 9) as usize;
    let vec_bytes = bytes.to_vec();
    let info_hash_range = info_hash_index..(info_hash_index + 20);
    let info_hash = vec_bytes[info_hash_range].to_vec();
    let peer_id_range = (info_hash_index + 20)..vec_bytes.len();
    let peer_id = vec_bytes[peer_id_range].to_vec();

    let string_peer_id = String::from_utf8_lossy(peer_id.as_ref()).to_string();
    HandShake::new(string_peer_id, info_hash.try_into().unwrap())
}

pub fn parse_have(bytes: [u8; 6]) -> Have {
    Have::new(bytes[5])
}
/*
pub fn parse_bitfield(bytes: Vec<u8>) -> Bitfield<'static> {
    //tengo 4 posiciones para len + 1 para id por lo que el bitfield empieza en la sexta posicion
    //lo que hago aca es obtener un slice del vector bytes que va desde 6 hasta len-1 y lo convierto en un nuevo vector
    let bitfield_start: usize = 5;
    let bitfield_end: usize = usize::from(bytes[0..3]) - 1;
    let bitfield = Vec::from(&bytes[bitfield_start..bitfield_end]);
    Bitfield::new(bytes[3], bitfield)
}
*/
pub fn is_handshake_message(bytes: [u8; 69]) -> bool {
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

pub fn is_have_message(bytes: [u8; 6]) -> bool {
    bytes[4] == 4
}
/*
pub fn is_bitfield_message(bytes: Vec<u8>) -> bool {
    bytes[4] == 5
}
pub fn is_interested_message(bytes: [u8; 5]) -> bool {
    bytes[4] == 2
}
pub fn is_unchoke_message(bytes: [u8; 5]) -> bool {
    bytes[4] == 1
}
*/
