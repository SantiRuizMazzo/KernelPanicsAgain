use super::download::peer::Peer;

#[derive(PartialEq, Eq, Debug)]
pub enum TrackerInfoState {
    Set(TrackerInfo),
    Unset,
}

impl TrackerInfoState {
    pub fn get_peers(&self) -> Option<Vec<Peer>> {
        match self {
            Self::Set(tracker_info) => Some(tracker_info.get_peers()),
            Self::Unset => None,
        }
    }
}

/// Stores the information received in a tracker response message.
#[derive(PartialEq, Eq, Debug)]
pub struct TrackerInfo {
    interval: u32,
    peers: Vec<Peer>,
}

impl TrackerInfo {
    pub fn new(interval: u32, peers: Vec<Peer>) -> TrackerInfo {
        TrackerInfo { interval, peers }
    }

    pub fn get_peers(&self) -> Vec<Peer> {
        self.peers.clone()
    }
}
