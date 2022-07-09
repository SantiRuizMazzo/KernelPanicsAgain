use super::{
    download_worker::{DownloadMessage, DownloadWorker},
    peer::Peer,
};
use crate::{
    client::{
        client_side::{DownloadedTorrents, TorrentReceiver, TorrentSender},
        torrent_piece::TorrentPiece,
    },
    config::Config,
    logger::torrent_logger::LogMessage,
};
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};

pub type PeerSender = Sender<Peer>;
pub type PeerReceiver = Arc<Mutex<Receiver<Peer>>>;
pub type PieceSender = Sender<DownloadMessage>;
pub type PieceReceiver = Arc<Mutex<Receiver<DownloadMessage>>>;
pub type DownloadedPieces = Arc<Mutex<Vec<TorrentPiece>>>;
pub type PeerBlacklist = Arc<Mutex<Vec<Peer>>>;

pub struct DownloadPool {
    workers: Vec<DownloadWorker>,
}

impl DownloadPool {
    pub fn new(
        torrent_queue: (TorrentSender, TorrentReceiver),
        downloaded_torrents: DownloadedTorrents,
        logger_tx: Sender<LogMessage>,
        client_id: [u8; 20],
        config: &Config,
    ) -> DownloadPool {
        let mut workers = Vec::with_capacity(config.get_max_download_connections());

        let _ = DownloadWorker::new(
            0,
            torrent_queue.clone(),
            downloaded_torrents.clone(),
            logger_tx.clone(),
            client_id,
            config,
        )
        .get_thread();

        for id in 0..workers.capacity() {
            workers.push(DownloadWorker::new(
                id,
                torrent_queue.clone(),
                downloaded_torrents.clone(),
                logger_tx.clone(),
                client_id,
                config,
            ));
        }

        DownloadPool { workers }
    }

    pub fn ids(&self) {
        for x in &self.workers {
            println!("{}", x.get_id())
        }
    }
}

pub fn setup_pieces_queue(pieces: &[TorrentPiece]) -> Result<(PieceSender, PieceReceiver), String> {
    let (piece_tx, piece_rx) = mpsc::channel::<DownloadMessage>();
    let piece_rx = Arc::new(Mutex::new(piece_rx));

    for &piece in pieces {
        piece_tx
            .send(DownloadMessage::Piece(piece))
            .map_err(|err| err.to_string())?;
    }
    Ok((piece_tx, piece_rx))
}
