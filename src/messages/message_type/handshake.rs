use std::io::Write;
use std::net::TcpStream;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HandShake<'a> {
    pstrlen: &'a [u8],
    pstr: String,
    reserved: &'a [u8],
    peer_id: String,
    info_hash: [u8; 20],
}

impl HandShake<'_> {
    pub fn new(peer_id: String, info_hash: [u8; 20]) -> HandShake<'static> {
        HandShake {
            pstrlen: &[19],
            pstr: "BitTorrent protocol".to_string(),
            reserved: &[0, 0, 0, 0, 0, 0, 0, 0],
            peer_id,
            info_hash,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(self.pstrlen)?;
        stream.write_all(self.pstr.as_bytes())?;
        stream.write_all(self.reserved)?;
        stream.write_all(&self.info_hash)?;
        stream.write_all(self.peer_id.as_bytes())?;
        stream.write_all("\n".as_bytes())?;
        Ok(())
    }

    pub fn set_peer_id(&mut self, peer_id: String) {
        self.peer_id = peer_id;
    }

    pub fn has_same_peer_id(&self, peer_id: Option<String>) -> bool {
        match peer_id {
            Some(id) => self.peer_id == id,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correctly_handshake() {
        let empty_array = [0; 20];
        let peer_id = "-PK0001-144591253628".to_string();
        let reserved = [0; 8];
        let handshake = HandShake::new(peer_id, empty_array);

        assert_eq!(1, handshake.pstrlen.len());
        assert_eq!("BitTorrent protocol".to_string(), handshake.pstr);
        assert_eq!("-PK0001-144591253628".to_string(), handshake.peer_id);
        assert_eq!(reserved, handshake.reserved);
        assert_eq!(empty_array, handshake.info_hash);
    }

    #[test]
    fn generate_correctly_handshake_size() {
        let empty_array = [0; 20];
        let peer_id = "-PK0001-144591253628".to_string();

        let handshake = HandShake::new(peer_id, empty_array);

        assert_eq!(
            68,
            handshake.pstrlen.len()
                + handshake.pstr.len()
                + handshake.peer_id.len()
                + handshake.reserved.len()
                + handshake.info_hash.len()
        );
    }
}
