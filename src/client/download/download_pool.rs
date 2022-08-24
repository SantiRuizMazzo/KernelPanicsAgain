use super::{download_worker::DownloadWorker, peer::Peer};
use crate::{
    client::client_side::{DownloadedTorrents, TorrentReceiver, TorrentSender},
    client::piece::Piece,
    config::Config,
    logging::log_handle::LogHandle,
    server::server_side::Notification,
};
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};

pub enum DownloadMessage {
    Piece(Piece),
    Kill,
}

pub type PeerSender = Sender<Peer>;
pub type PeerReceiver = Arc<Mutex<Receiver<Peer>>>;
pub type PieceSender = Sender<DownloadMessage>;
pub type PieceReceiver = Arc<Mutex<Receiver<DownloadMessage>>>;
pub type DownloadedPieces = Arc<Mutex<Vec<Piece>>>;

pub struct DownloadPool {
    workers: Vec<DownloadWorker>,
}

impl DownloadPool {
    pub fn new(
        client_id: [u8; 20],
        config: &Config,
        (torrent_tx, torrent_rx_mutex): (&TorrentSender, &TorrentReceiver),
        downloaded_torrents: &DownloadedTorrents,
        notif_tx: Sender<Notification>,
        log_handle: &LogHandle,
    ) -> Self {
        let mut workers = Vec::with_capacity(config.get_max_download_connections());

        for id in 0..workers.capacity() {
            workers.push(DownloadWorker::new(
                id,
                client_id,
                config.torrent_time_slice(),
                (torrent_tx.clone(), torrent_rx_mutex.clone()),
                downloaded_torrents.clone(),
                notif_tx.clone(),
                log_handle.clone(),
            ));
        }

        Self { workers }
    }
    pub fn wait_for_workers(&mut self) {
        for worker in &mut self.workers {
            let _ = worker.join();
        }
    }
}

impl Drop for DownloadPool {
    fn drop(&mut self) {
        self.wait_for_workers()
    }
}
