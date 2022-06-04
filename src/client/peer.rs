/// Stores information about each peer in the peer list that is provided by the tracker.
#[derive(PartialEq, Eq, Debug)]
pub struct Peer {
    id: Option<[u8; 20]>,
    ip: String,
    port: u32,
}

impl Peer {
    pub fn new(id: Option<[u8; 20]>, ip: String, port: u32) -> Peer {
        Peer { id, ip, port }
    }

    //TEMPORAL FUNCTION TO FIX CLIPPY WARNINGS!
    pub fn print(&self) {
        println!("{}, {}", self.ip, self.port)
    }
}
