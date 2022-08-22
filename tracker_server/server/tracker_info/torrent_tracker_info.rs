use std::collections::HashMap;

use super::peer_tracker_info::PeerTrackerInfo;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TorrentTrackerData {
    info_hash: String,
    peers: HashMap<String, PeerTrackerInfo>,
}

impl Default for TorrentTrackerData {
    fn default() -> Self {
        Self {
            info_hash: String::new(),
            peers: HashMap::new(),
        }
    }
}

impl TorrentTrackerData {
    pub fn new(info_hash: String, peers: HashMap<String, PeerTrackerInfo>) -> TorrentTrackerData {
        TorrentTrackerData { info_hash, peers }
    }

    pub fn insert_peer(&mut self, peer: PeerTrackerInfo) {
        self.peers.insert(peer.clone().get_peer_id(), peer);
    }

    pub fn get_peers_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<Vec<u8>> = Vec::new();
        self.peers
            .iter()
            .for_each(|(_peer_id, peer_state)| vec.push(peer_state.get_peer_address_bytes()));
        vec.into_iter().flatten().collect()
    }

    pub fn get_connected_peers(&self) -> u32 {
        let mut count = 0;
        self.peers.iter().for_each(|(_, peer_state)| {
            count += peer_state.get_connected_value();
        });
        count
    }

    pub fn get_completed_peers(&self) -> u32 {
        let mut count = 0;
        self.peers.iter().for_each(|(_, peer_state)| {
            count += peer_state.get_completed_value();
        });
        count
    }
}
