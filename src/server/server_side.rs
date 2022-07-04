pub struct ServerSide {
    peer_id: [u8; 20],
}

impl ServerSide {
    pub fn new() -> ServerSide {
        ServerSide { peer_id: [0; 20] }
    }

    pub fn set_peer_id(&mut self, peer_id: [u8; 20]) {
        self.peer_id = peer_id;
    }
}
