use super::{torrent_upload_info::UploadInfo, upload_worker::UploadWorker};
use crate::{
    client::{download::peer::Peer, torrent_piece::TorrentPiece},
    messages::message_type::handshake::HandShake, logger::torrent_logger::LogMessage,
};
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub enum CleanerMessage {
    AddWorker(TcpStream, HandShake),
    RemoveWorker(usize),
    Kill,
}

pub struct UploadPool {
    worker_cleaner: Option<JoinHandle<Result<(), String>>>,
    cleaner_tx: Sender<CleanerMessage>,
    offered_torrents: Arc<Mutex<HashMap<[u8; 20], UploadInfo>>>,
}

impl UploadPool {
    pub fn new(logger_tx: Sender<LogMessage>) -> UploadPool {
        let offered_torrents = Arc::new(Mutex::new(HashMap::<[u8; 20], UploadInfo>::new()));

        let (cleaner_tx, cleaner_rx) = mpsc::channel::<CleanerMessage>();
        let offered_torrents_clone = offered_torrents.clone();
        let cleaner_tx_for_worker = cleaner_tx.clone();

        let worker_cleaner = thread::spawn(move || {
            let mut workers = HashMap::<usize, UploadWorker>::new();

            loop {
                match cleaner_rx.recv().map_err(|err| err.to_string())? {
                    CleanerMessage::AddWorker(stream, received_handshake) => {
                        let peer_addr = stream.peer_addr().map_err(|error| error.to_string())?;
                        let ip = peer_addr.ip().to_string();
                        let port = peer_addr.port() as u32;
                        let peer = Peer::new(Some(received_handshake.get_peer_id()), ip, port, 0);

                        let new_worker = UploadWorker::new(
                            stream,
                            peer,
                            received_handshake.get_info_hash(),
                            offered_torrents_clone.clone(),
                            cleaner_tx_for_worker.clone(),
                            workers.len(),
                            logger_tx.clone()
                        );

                        let _ = workers.insert(workers.len(), new_worker);
                    }
                    CleanerMessage::RemoveWorker(key) => {
                        UploadPool::remove_worker(&mut workers, key)
                    }
                    CleanerMessage::Kill => {
                        for (_, mut worker) in workers.drain() {
                            if let Some(thread) = worker.get_thread() {
                                let _ = thread.join();
                            }
                        }
                        break;
                    }
                }
            }

            Ok(())
        });

        UploadPool {
            worker_cleaner: Some(worker_cleaner),
            cleaner_tx,
            offered_torrents,
        }
    }

    fn remove_worker(workers: &mut HashMap<usize, UploadWorker>, worker_id: usize) {
        if let Some(mut worker) = workers.remove(&worker_id) {
            if let Some(thread) = worker.get_thread() {
                let _ = thread.join();
            }
        }
    }

    pub fn add_new_connection(
        &mut self,
        stream: TcpStream,
        received_handshake: HandShake,
    ) -> Result<(), String> {
        self.cleaner_tx
            .send(CleanerMessage::AddWorker(stream, received_handshake))
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn offer_new_piece(
        &mut self,
        piece: TorrentPiece,
        upload_info: UploadInfo,
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

    pub fn is_serving(&self, info_hash: [u8; 20]) -> Result<bool, String> {
        let offered = self
            .offered_torrents
            .lock()
            .map_err(|err| err.to_string())?;

        Ok(offered.contains_key(&info_hash))
    }
}

impl Drop for UploadPool {
    fn drop(&mut self) {
        let _ = self
            .cleaner_tx
            .send(CleanerMessage::Kill)
            .map_err(|error| error.to_string());
        if let Some(thread) = self.worker_cleaner.take() {
            let _ = thread.join();
        }
    }
}
