use super::metadata::Metadata;
use super::peer_state::PeerState;

impl Default for TorrentState {
    fn default() -> Self {
        Self::new(0)
    }
}
impl TorrentState {
    pub fn new(total_peers: usize) -> TorrentState {
        TorrentState {
            peers: Vec::new(),
            metadata: Metadata::new(),
            total_peers,
        }
    }

    pub fn add_peer_state(&mut self, state: PeerState) {
        self.peers.push(state);
    }

    pub fn set_metadata_info_hash(&mut self, _info_hash: [u8; 20]) {
        self.metadata.set_info_hash(_info_hash);
    }

    pub fn get_metadata_info_hash(&self) -> [u8; 20] {
        self.metadata.get_info_hash()
    }

    pub fn set_metadata_total_size(&mut self, total_size: u32) {
        self.metadata.set_total_size(total_size);
    }

    pub fn get_metadata_total_size(&self) -> u32 {
        self.metadata.get_total_size()
    }

    pub fn set_metadata_n_pieces(&mut self, n_pieces: u32) {
        self.metadata.set_n_pieces(n_pieces);
    }

    pub fn get_metadata_n_pieces(&self) -> u32 {
        self.metadata.get_n_pieces()
    }

    pub fn set_metadata_is_single(&mut self, _is_single: bool) {
        self.metadata.set_is_single(_is_single);
    }

    pub fn get_metadata_is_single(&self) -> bool {
        self.metadata.get_is_single()
    }

    pub fn set_metadata_downloaded(&mut self, _downloaded: u32) {
        self.metadata.set_downloaded(_downloaded);
    }

    pub fn get_metadata_downloaded(&self) -> u32 {
        self.metadata.get_downloaded()
    }

    pub fn set_metadata_connections(&mut self, _conections: usize) {
        self.metadata.set_connections(_conections as u32);
    }

    pub fn get_metadata_connections(&self) -> u32 {
        self.metadata.get_connections()
    }

    pub fn set_metadata_name(&mut self, _name: String) {
        self.metadata.set_name(_name);
    }

    pub fn get_metadata_name(&self) -> String {
        self.metadata.get_name()
    }

    pub fn get_metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    pub fn get_peers(&self) -> Vec<PeerState> {
        self.peers.clone()
    }

    pub fn get_total_peers(&self) -> usize {
        self.total_peers.clone()
    }

    pub fn get_completion_precentage(&self) -> f64 {
        ((self.get_metadata_downloadede() as f64)/(self.get_metadata_n_pieces() as f64)) * 100.0
    }
}

#[derive(Clone)]
pub struct TorrentState {
    peers: Vec<PeerState>,
    metadata: Metadata,
    total_peers: usize,
}
