use super::{
    super::{
        bdecoding::{BDecoder, BType},
        utils,
    },
    peer::Peer,
    tracker_info::TrackerInfo,
};
use std::collections::HashMap;

pub fn tracker_info_from_bytes(bytes: Vec<u8>) -> Result<TrackerInfo, String> {
    let body = match BDecoder::bdecode(bytes)? {
        BType::Dictionary(body) => body,
        _ => return Err("tracker response body is not a bencoded dictionary".to_string()),
    };

    let interval = match body.get("interval") {
        Some(BType::Integer(interval)) => *interval as u32,
        _ => return Err("interval key not present or has invalid value type".to_string()),
    };

    let peers = match body.get("peers") {
        Some(BType::List(peers)) => regular_peer_list(peers)?,
        Some(BType::String(peers)) => compact_peer_list(peers),
        _ => return Err("peers key not present or has invalid value type".to_string()),
    };

    Ok(TrackerInfo::new(interval, peers))
}

fn regular_peer_list(peers: &[BType]) -> Result<Vec<Peer>, String> {
    let mut peer_list = Vec::with_capacity(peers.len());
    for peer in peers {
        match peer {
            BType::Dictionary(peer_dict) => peer_list.push(peer_from_dict(peer_dict)?),
            _ => return Err("some peer of the list is not a bencoded dictionary".to_string()),
        }
    }

    if peer_list.is_empty() {
        return Err("could not load any peers to the peers list".to_string());
    }
    Ok(peer_list)
}

fn peer_from_dict(peer_dict: &HashMap<String, BType>) -> Result<Peer, String> {
    let id = match peer_dict.get("peer id") {
        Some(BType::String(id)) => peer_id_from_bytes(id),
        _ => None,
    };

    let ip = match peer_dict.get("ip") {
        Some(BType::String(ip)) => utils::bytes_to_string(ip)?,
        _ => return Err("ip key not present in peer dictionary".to_string()),
    };

    let port = match peer_dict.get("port") {
        Some(BType::Integer(port)) => *port as u32,
        _ => return Err("port key not present in peer dictionary".to_string()),
    };

    Ok(Peer::new(id, ip, port))
}

fn peer_id_from_bytes(peer_id: &[u8]) -> Option<[u8; 20]> {
    if peer_id.len() != 20 {
        return None;
    }
    peer_id.try_into().ok()
}

fn compact_peer_list(peers: &[u8]) -> Vec<Peer> {
    let mut peer_list = Vec::with_capacity(peers.len() / 6);
    for peer in peers.chunks_exact(6) {
        let ip = format!("{}.{}.{}.{}", peer[0], peer[1], peer[2], peer[3]);
        let port = u32::from_be_bytes([0, 0, peer[4], peer[5]]);
        peer_list.push(Peer::new(None, ip, port));
    }
    peer_list
}
