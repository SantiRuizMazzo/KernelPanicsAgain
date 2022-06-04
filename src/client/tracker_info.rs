use super::peer::Peer;

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

    //TEMPORAL FUNCTION TO FIX CLIPPY WARNINGS!
    pub fn print(&self) {
        println!("{}, {:?}", self.interval, self.peers)
    }
}
