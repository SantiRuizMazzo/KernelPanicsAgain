use super::{peer::Peer, torrent_decoder::TorrentDecoder};
use crate::urlencoding::encode;
use rand::Rng;

pub struct ClientSide {
    pub peer_id: String,
}
impl ClientSide {
    fn generate_peer_id() -> String {
        let mut peer_id = String::from("-PK0001-");
        let mut generator = rand::thread_rng();
        for _i in 0..12 {
            let aux: i8 = generator.gen_range(0..10);
            peer_id += &aux.to_string();
        }
        peer_id
    }
    pub fn new() -> ClientSide {
        let _ = TorrentDecoder::decode("tests/sample.torrent");
        let peer = Peer::new("hola".to_string(), "chau".to_string(), 0);
        peer.print();
        let _ = encode("上海+中國");

        ClientSide {
            peer_id: ClientSide::generate_peer_id(),
        }
    }
}

/*-<2 letras><4 numeros de version>-<12 numeros random>*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correctly_sized_peer_id() {
        let s = ClientSide::generate_peer_id();
        assert_eq!(20, s.len() * std::mem::size_of::<u8>());
    }

    #[test]
    fn generate_correctly_sized_peer_id_inside_clientside_struct() {
        let client = ClientSide::new();
        assert_eq!(20, client.peer_id.len() * std::mem::size_of::<u8>());
    }
}
