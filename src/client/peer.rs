/// Stores information about each peer in the peer list that is provided by the tracker.

pub struct Peer {
    peer_id: String,
    ip: String,
    port: i32,
}

impl Peer {
    pub fn new(peer_id: String, ip: String, port: i32) -> Peer {
        Peer { peer_id, ip, port }
    }

    pub fn print(&self) {
        println!("{}, {}, {}", self.peer_id, self.ip, self.port)
    }
}
