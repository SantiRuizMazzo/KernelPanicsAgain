/// Stores information about each peer in the peer list that is provided by the tracker.
#[derive(Debug)]
pub struct Peer {
    ip: String,
    port: u32,
}

impl Peer {
    pub fn new(ip: String, port: u32) -> Peer {
        Peer { ip, port }
    }

    //TEMPORAL FUNCTION TO FIX CLIPPY WARNINGS!
    pub fn print(&self) {
        println!("{}, {}", self.ip, self.port)
    }
}
