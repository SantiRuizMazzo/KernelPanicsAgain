use super::{
    download_info::DownloadInfo,
    download_worker::{DownloadMessage, DownloadWorker},
    peer::Peer,
};
use crate::{client::torrent_piece::TorrentPiece, logger::torrent_logger::LogMessage};
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};

type PeerSender = Sender<Peer>;
type PeerReceiver = Arc<Mutex<Receiver<Peer>>>;
type PieceSender = Sender<DownloadMessage>;
type PieceReceiver = Arc<Mutex<Receiver<DownloadMessage>>>;

pub struct DownloadPool {
    workers: Vec<DownloadWorker>,
    piece_tx: Sender<DownloadMessage>,
    kill_rx: Receiver<DownloadMessage>,
}

impl DownloadPool {
    pub fn new(
        size: usize,
        pieces: &[TorrentPiece],
        peers: &[Peer],
        logger_tx: Sender<LogMessage>,
        download: DownloadInfo,
    ) -> Result<DownloadPool, String> {
        let pieces_handle = setup_pieces_queue(pieces)?;
        let peers_handle = setup_peers_queue(peers)?;
        let kill_handle = mpsc::channel::<DownloadMessage>();

        let downloaded = Arc::new(Mutex::new(Vec::<TorrentPiece>::with_capacity(pieces.len())));
        let blacklist = Arc::new(Mutex::new(Vec::<Peer>::with_capacity(peers.len())));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..workers.capacity() {
            workers.push(DownloadWorker::new(
                download.clone(),
                pieces_handle.clone(),
                peers_handle.clone(),
                blacklist.clone(),
                downloaded.clone(),
                kill_handle.0.clone(),
                logger_tx.clone(),
            ));
        }

        Ok(DownloadPool {
            workers,
            piece_tx: pieces_handle.0,
            kill_rx: kill_handle.1,
        })
    }
}

fn setup_pieces_queue(pieces: &[TorrentPiece]) -> Result<(PieceSender, PieceReceiver), String> {
    let (piece_tx, piece_rx) = mpsc::channel::<DownloadMessage>();
    let piece_rx = Arc::new(Mutex::new(piece_rx));

    for &piece in pieces {
        piece_tx
            .send(DownloadMessage::Piece(piece))
            .map_err(|err| err.to_string())?;
    }
    Ok((piece_tx, piece_rx))
}

fn setup_peers_queue(peers: &[Peer]) -> Result<(PeerSender, PeerReceiver), String> {
    let (peer_tx, peer_rx) = mpsc::channel::<Peer>();
    let peer_rx = Arc::new(Mutex::new(peer_rx));

    for peer in peers {
        peer_tx.send(peer.clone()).map_err(|err| err.to_string())?;
    }
    Ok((peer_tx, peer_rx))
}

impl Drop for DownloadPool {
    fn drop(&mut self) {
        let _ = self.kill_rx.recv();

        for _ in &self.workers {
            let _ = self.piece_tx.send(DownloadMessage::Kill);
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.get_thread() {
                let _ = thread.join();
            }
        }
    }
}
