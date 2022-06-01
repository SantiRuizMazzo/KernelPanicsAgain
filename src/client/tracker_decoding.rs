use super::{
    super::{
        bdecoding::{BDecoder, BType},
        utils::bytes_to_string,
    },
    peer::Peer,
    tracker_info::TrackerInfo,
};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

///

pub fn tracker_info_from_bytes(bytes: Vec<u8>) -> Result<TrackerInfo, Error> {
    let btype = BDecoder::bdecode(bytes)?;
    get_tracker_info(btype)
}

///

fn get_tracker_info(btype: BType) -> Result<TrackerInfo, Error> {
    let body = match btype {
        BType::Dictionary(body) => body,
        _ => return error("tracker response body is not a bencoded dictionary"),
    };

    let interval = match body.get("interval") {
        Some(BType::Integer(interval)) => *interval,
        _ => return error("interval key not present or has invalid value type"),
    };

    let peers = match body.get("peers") {
        Some(BType::List(peers)) => peer_list(peers)?,
        Some(BType::String(peers)) => compact_peer_list(peers)?,
        _ => return error("interval key not present or has invalid value type"),
    };

    Ok(TrackerInfo::new(interval as u32, peers))
}

///

fn peer_list(btype_list: &[BType]) -> Result<Vec<Peer>, Error> {
    let mut peer_list: Vec<Peer> = Vec::with_capacity(btype_list.len());
    if btype_list.is_empty() {
        return error("peer list does not have any peers");
    }

    for btype in btype_list {
        let peer = match btype {
            BType::Dictionary(peer_dict) => peer_from_dict(peer_dict)?,
            _ => return error("some peer of the list is not a bencoded dictionary"),
        };
        peer_list.push(peer);
    }
    Ok(peer_list)
}

///

fn peer_from_dict(peer_dict: &HashMap<String, BType>) -> Result<Peer, Error> {
    let ip = match peer_dict.get("ip") {
        Some(BType::String(ip)) => bytes_to_string(ip)?,
        _ => return error("ip key not present in peer dictionary"),
    };

    let port = match peer_dict.get("port") {
        Some(BType::Integer(port)) => *port,
        _ => return error("port key not present in peer dictionary"),
    };

    Ok(Peer::new(ip, port as u32))
}

///

fn compact_peer_list(peers: &[u8]) -> Result<Vec<Peer>, Error> {
    let mut peer_list: Vec<Peer> = Vec::with_capacity(peers.len() / 6);

    for peer in peers.chunks_exact(6) {
        let ip = format!("{}.{}.{}.{}", peer[0], peer[1], peer[2], peer[3]);
        let port = u32::from_be_bytes([0, 0, peer[4], peer[5]]);
        peer_list.push(Peer::new(ip, port));
    }

    Ok(peer_list)
}

/// Creates an instance of an `io::Error` with a specificied `msg`.

fn error<T>(msg: &str) -> Result<T, Error> {
    Err(Error::new(ErrorKind::Other, msg))
}
