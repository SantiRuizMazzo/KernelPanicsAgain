use std::net::Ipv4Addr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PeerTrackerState {
    Started,
    Stopped,
    Completed,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PeerTrackerInfo {
    peer_id: String,
    peer_ip: Ipv4Addr,
    port: u16,
    state: PeerTrackerState,
}
impl PeerTrackerInfo {
    pub fn new(
        peer_id: String,
        peer_ip: Ipv4Addr,
        port: u16,
        state: PeerTrackerState,
    ) -> PeerTrackerInfo {
        PeerTrackerInfo {
            peer_id,
            peer_ip,
            port,
            state,
        }
    }
    pub fn get_peer_id(&self) -> String {
        self.peer_id.clone()
    }

    pub fn get_connected_value(&self) -> u32 {
        match self.state {
            PeerTrackerState::Stopped => 0,
            _ => 1,
        }
    }

    pub fn get_completed_value(&self) -> u32 {
        match self.state {
            PeerTrackerState::Completed => 1,
            _ => 0,
        }
    }
    pub fn get_peer_address_bytes(&self) -> Vec<u8> {
        let mut address_bytes = self.peer_ip.octets().to_vec();
        let mut port_bytes = u16::to_be_bytes(self.port).to_vec();
        address_bytes.append(&mut port_bytes);
        address_bytes
    }
}

#[test]
fn test_peer_address_bytes() {
    let peer = PeerTrackerInfo::new(
        "laksjflkasj".to_string(),
        Ipv4Addr::new(192, 168, 3, 4),
        8080,
        PeerTrackerState::Started,
    );
    let address_bytes: Vec<u8> = vec![192, 168, 3, 4, 31, 144];
    assert_eq!(address_bytes, peer.get_peer_address_bytes())
}
