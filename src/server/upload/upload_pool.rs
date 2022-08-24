use super::{upload_info::UploadInfo, upload_worker::UploadWorker};
use crate::{client::piece::Piece, server::server_side::Notification};
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{mpsc::Sender, Arc, Mutex},
};

pub struct UploadPool {
    server_id: [u8; 20],
    torrents: Arc<Mutex<HashMap<[u8; 20], UploadInfo>>>,
    workers: HashMap<usize, UploadWorker>,
}

impl Default for UploadPool {
    fn default() -> Self {
        Self::new([0; 20])
    }
}

impl UploadPool {
    pub fn new(server_id: [u8; 20]) -> Self {
        Self {
            server_id,
            torrents: Arc::new(Mutex::new(HashMap::<[u8; 20], UploadInfo>::new())),
            workers: HashMap::<usize, UploadWorker>::new(),
        }
    }

    pub fn add_worker(
        &mut self,
        stream: TcpStream,
        notif_tx: &Sender<Notification>,
    ) -> Result<(), String> {
        let worker = UploadWorker::new(
            self.workers.len(),
            stream,
            self.server_id,
            self.torrents.clone(),
            notif_tx.clone(),
        )?;

        self.workers.insert(worker.id(), worker);
        Ok(())
    }

    pub fn remove_worker(&mut self, id: usize) -> Result<(), String> {
        self.workers
            .remove(&id)
            .ok_or_else(|| format!("Error removing upload worker {id}"))?
            .join()
    }

    pub fn add_piece(&mut self, piece: Piece, upload_info: UploadInfo) -> Result<(), String> {
        let mut torrents = self.torrents.lock().map_err(|e| e.to_string())?;

        let mut updated_info = match torrents.remove(&upload_info.info_hash()) {
            Some(info) => info,
            None => upload_info,
        };

        updated_info.add_piece_to_bitfield(piece.index())?;
        torrents.insert(updated_info.info_hash(), updated_info);
        Ok(())
    }
}

impl Drop for UploadPool {
    fn drop(&mut self) {
        for (_, mut worker) in self.workers.drain() {
            let _ = worker.join();
        }
    }
}
