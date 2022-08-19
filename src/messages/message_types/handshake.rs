use crate::client::download::peer_protocol::ProtocolError;
use std::{
    io::{Error, Write},
    net::TcpStream,
};

pub const HANDSHAKE_PSTR: &str = "BitTorrent protocol";

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Handshake {
    pstrlen: u8,
    pstr: String,
    reserved: [u8; 8],
    peer_id: [u8; 20],
    info_hash: [u8; 20],
}

impl Handshake {
    pub fn new(pstr: &str, reserved: [u8; 8], info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            pstrlen: pstr.len() as u8,
            pstr: pstr.to_string(),
            reserved,
            peer_id,
            info_hash,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
        stream.write_all(&self.pstrlen.to_be_bytes()).map_err(err)?;
        stream.write_all(self.pstr.as_bytes()).map_err(err)?;
        stream.write_all(&self.reserved).map_err(err)?;
        stream.write_all(&self.info_hash).map_err(err)?;
        stream.write_all(&self.peer_id).map_err(err)
    }

    pub fn info_hash(&self) -> [u8; 20] {
        self.info_hash
    }
    pub fn peer_id(&self) -> [u8; 20] {
        self.peer_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correctly_handshake() {
        let empty_array = [0; 20];
        let peer_id = *b"-PK0001-144591253628";
        let reserved = [0; 8];
        let handshake = Handshake::new(HANDSHAKE_PSTR, reserved, empty_array, peer_id);

        assert_eq!(19, handshake.pstrlen);
        assert_eq!(HANDSHAKE_PSTR, handshake.pstr);
        assert_eq!(*b"-PK0001-144591253628", handshake.peer_id);
        assert_eq!(reserved, handshake.reserved);
        assert_eq!(empty_array, handshake.info_hash);
    }

    #[test]
    fn generate_correctly_handshake_size() {
        let empty_array = [0; 20];
        let peer_id = *b"-PK0001-144591253628";
        let reserved = [0; 8];
        let handshake = Handshake::new(HANDSHAKE_PSTR, reserved, peer_id, empty_array);

        assert_eq!(
            68,
            1 + handshake.pstr.len()
                + handshake.peer_id.len()
                + handshake.reserved.len()
                + handshake.info_hash.len()
        );
    }
}
