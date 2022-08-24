use std::time::Instant;

use crate::ui_notification_structs::{peer_state::PeerState, torrent_state::TorrentState};

use super::peer::Peer;

impl DownloadWorkerState {
    pub fn new(
        id: usize,
        info_hash: [u8; 20],
        current_peer: Peer,
        torrent_name: String,
        n_pieces: usize,
        last_download_instant: Option<Instant>,
        total_peers: usize,
    ) -> DownloadWorkerState {
        DownloadWorkerState {
            id,
            info_hash,
            curr_peer_id: current_peer.id(),
            peer_ip: current_peer.ip(),
            peer_port: current_peer.port(),
            am_interested: true,
            am_choked: true,
            is_interested: true,
            is_choked: true,
            torrent_name,
            n_pieces,
            downloaded_pieces: 0,
            is_single: true,
            total_size: 0,
            last_download_instant,
            total_peers,
        }
    }

    pub fn set_am_interested(&mut self, interested: bool) {
        self.am_interested = interested;
    }

    pub fn set_am_choked(&mut self, choked: bool) {
        self.am_choked = choked;
    }

    pub fn set_is_interested(&mut self, interested: bool) {
        self.is_interested = interested;
    }

    pub fn set_is_choked(&mut self, choked: bool) {
        self.is_choked = choked;
    }

    pub fn is_choked(&self) -> bool {
        self.is_choked
    }

    pub fn is_interested(&self) -> bool {
        self.is_interested
    }
    pub fn set_total_size(&mut self, size: u32) {
        self.total_size = size as usize;
    }
    pub fn set_downloaded_pieces(&mut self, downloaded_pieces: usize) {
        self.downloaded_pieces = downloaded_pieces;
    }
    pub fn generate_torrent_state(
        &self,
        connections_qty: usize,
        peer_states: Vec<PeerState>,
    ) -> TorrentState {
        let mut state = TorrentState::new(self.total_peers);
        state.set_metadata_connections(connections_qty);
        state.set_metadata_downloaded(self.downloaded_pieces as u32);
        state.set_metadata_info_hash(self.info_hash);
        state.set_metadata_is_single(self.is_single);
        state.set_metadata_n_pieces(self.n_pieces as u32);
        state.set_metadata_name(self.torrent_name.clone());
        state.set_metadata_total_size(self.total_size as u32);
        peer_states
            .iter()
            .for_each(|peer_state| state.add_peer_state(peer_state.clone()));
        state
    }
    pub fn generate_peer_state(&self) -> PeerState {
        let mut peer_state = PeerState::new(self.last_download_instant);
        peer_state.set_id(format!("{:x?}", self.curr_peer_id));
        peer_state.set_port(self.peer_port);
        peer_state.set_ip(self.peer_ip.clone());
        peer_state.set_c_is_chocked(self.am_choked);
        peer_state.set_c_is_interested(self.am_interested);

        peer_state.set_p_is_chocked(self.is_choked);
        peer_state.set_p_is_interested(self.is_interested);
        peer_state
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DownloadWorkerState {
    pub id: usize,
    pub info_hash: [u8; 20],
    pub curr_peer_id: Option<[u8; 20]>,
    pub peer_ip: String,
    pub peer_port: u16,
    pub am_interested: bool,
    pub am_choked: bool,
    pub is_interested: bool,
    pub is_choked: bool,
    pub torrent_name: String,
    pub n_pieces: usize,
    pub downloaded_pieces: usize,
    pub is_single: bool,
    pub total_size: usize,
    pub last_download_instant: Option<Instant>,
    pub total_peers: usize,
}
