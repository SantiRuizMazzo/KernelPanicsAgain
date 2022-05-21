use super::peer::Peer;

/// Stores the information received in a tracker response message.

pub struct TrackerInfo {
    pub interval: i32,
    pub complete: i32,
    pub incomplete: i32,
    pub tracker_id: String,
    pub peers: Vec<Peer>,
}
