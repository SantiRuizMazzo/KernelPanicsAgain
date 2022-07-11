use super::{torrent_upload_info::TorrentUploadInfo, upload_worker::UploadWorker};
use crate::{
    client::{download::peer::Peer, torrent_piece::TorrentPiece},
    messages::message_type::handshake::HandShake,
};
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub struct UploadPool {
    workers: Vec<UploadWorker>,
    offered_torrents: Arc<Mutex<HashMap<[u8; 20], TorrentUploadInfo>>>,
}

impl Default for UploadPool {
    fn default() -> Self {
        Self::new()
    }
}

impl UploadPool {
    pub fn new() -> UploadPool {
        let workers = Vec::<UploadWorker>::new();
        let offered_torrents = Arc::new(Mutex::new(HashMap::<[u8; 20], TorrentUploadInfo>::new()));
        // let mut worker_states =  Arc::new(Mutex::new(Vec::<WorkerState>::new()));
        UploadPool {
            workers,
            offered_torrents,
        }
    }

    pub fn add_new_connection(
        &mut self,
        stream: TcpStream,
        received_handshake: HandShake,
    ) -> Result<(), String> {
        let peer_addr = stream.peer_addr().map_err(|error| error.to_string())?;
        let ip = peer_addr.ip().to_string();
        let port = peer_addr.port() as u32;
        let peer = Peer::new(Some(received_handshake.get_peer_id()), ip, port, 0);

        self.workers.push(UploadWorker::new(
            stream,
            peer,
            received_handshake.get_info_hash(),
            self.offered_torrents.clone(),
        ));
        Ok(())
    }

    pub fn offer_new_piece(
        &mut self,
        piece: TorrentPiece,
        upload_info: TorrentUploadInfo,
    ) -> Result<(), String> {
        let mut offered = self
            .offered_torrents
            .lock()
            .map_err(|err| err.to_string())?;

        let mut info_to_insert = match offered.get_mut(&upload_info.get_hash()) {
            Some(found) => found.clone(),
            None => upload_info,
        };

        info_to_insert.add_piece_to_bitfield(piece.get_index())?;
        offered.insert(info_to_insert.get_hash(), info_to_insert.clone());

        Ok(())
    }

    pub fn is_torrent_offered(&self, info_hash: [u8; 20]) -> Result<bool, String> {
        let offered = self
            .offered_torrents
            .lock()
            .map_err(|err| err.to_string())?;

        if let Some(_upload_info) = offered.get(&info_hash) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Drop for UploadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread) = worker.get_thread() {
                let _ = thread.join();
            }
        }
    }
}
