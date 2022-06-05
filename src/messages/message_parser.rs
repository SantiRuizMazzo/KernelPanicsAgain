use crate::messages::message_type::{handshake::HandShake, have::Have};

pub fn parse_handshake(bytes: [u8; 68]) -> Result<HandShake<'static>, String> {
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

pub fn parse_have(bytes: [u8; 6]) -> Result<Have, String> {
    if !is_have_message(bytes) {
        return Err("received message is not a have".to_string());
    }
    Ok(Have::new(bytes[5]))
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
