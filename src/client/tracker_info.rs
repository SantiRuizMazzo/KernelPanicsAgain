use super::download::peer::Peer;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TrackerInfoState {
    Set(TrackerInfo),
    Unset,
}

impl TrackerInfoState {
    pub fn is_set(&self) -> bool {
        match self {
            Self::Set(_) => true,
            Self::Unset => false,
        }
    }
}

/// Stores the information received in a tracker response message.
#[derive(PartialEq, Eq, Debug, Clone)]
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
