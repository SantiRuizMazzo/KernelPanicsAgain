use super::{download_info::DownloadInfo, peer::Peer};
use crate::{
    client::{download::peer_protocol::DownloadError, torrent_piece::TorrentPiece},
    logger::torrent_logger::LogMessage,
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub enum DownloadMessage {
    Piece(TorrentPiece),
    Kill,
}

pub struct DownloadWorker {
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl DownloadWorker {
    pub fn new(
        download: DownloadInfo,
        (piece_tx, piece_rx_mutex): (
            Sender<DownloadMessage>,
            Arc<Mutex<Receiver<DownloadMessage>>>,
        ),
        (peer_tx, peer_rx_mutex): (Sender<Peer>, Arc<Mutex<Receiver<Peer>>>),
        blacklist_mutex: Arc<Mutex<Vec<Peer>>>,
        downloaded_mutex: Arc<Mutex<Vec<TorrentPiece>>>,
        kill_tx: Sender<DownloadMessage>,
        logger_tx: Sender<LogMessage>,
    ) -> DownloadWorker {
        let thread = thread::spawn(move || {
            let mut peer = Peer::new(Some(download.get_id()), "".to_string(), 80, 0);
            let mut connection = None;
            let mut current_piece = None;
            let mut piece;

            loop {
                let downloaded = downloaded_mutex
                    .lock()
                    .map_err(|_| "mutex lock error".to_string())?;
                let (current_pieces, total_pieces) = (downloaded.len(), downloaded.capacity());
                drop(downloaded);

                let progress = (current_pieces as f32 * 100_f32) / (total_pieces as f32);
                logger_tx
                    .send(LogMessage::Log(format!(
                        "Downloaded pieces: {current_pieces}/{total_pieces} ({progress:.2}%)"
                    )))
                    .map_err(|err| err.to_string())?;

                if current_pieces == total_pieces {
                    kill_tx
                        .send(DownloadMessage::Kill)
                        .map_err(|err| err.to_string())?;
                }

                if current_piece.is_none() {
                    let piece_rx = piece_rx_mutex
                        .lock()
                        .map_err(|_| "mutex lock error".to_string())?;
                    let message = piece_rx
                        .recv()
                        .map_err(|_| "piece queue error".to_string())?;
                    drop(piece_rx);

                    current_piece = match message {
                        DownloadMessage::Piece(piece) => Some(piece),
                        DownloadMessage::Kill => break,
                    };
                }

                let target_piece = match current_piece {
                    Some(piece) => piece,
                    None => continue,
                };

                if connection.is_none() {
                    let peer_rx = peer_rx_mutex
                        .lock()
                        .map_err(|_| "mutex lock error".to_string())?;
                    peer = peer_rx.recv().map_err(|_| "peer queue error".to_string())?;
                    drop(peer_rx);
                }

                (connection, piece) =
                    match peer.download(target_piece, connection, total_pieces, download.clone()) {
                        Ok((stream, piece)) => (Some(stream), piece),
                        Err(DownloadError::Connection(_)) => {
                            update_blacklist(&blacklist_mutex, &peer, &peer_tx)?;
                            connection = None;
                            continue;
                        }
                        Err(DownloadError::Piece(_)) => {
                            piece_tx
                                .send(DownloadMessage::Piece(target_piece))
                                .map_err(|err| err.to_string())?;
                            current_piece = None;
                            peer_tx.send(peer.clone()).map_err(|err| err.to_string())?;
                            connection = None;
                            continue;
                        }
                    };

                let mut downloaded = downloaded_mutex
                    .lock()
                    .map_err(|_| "mutex lock error".to_string())?;
                downloaded.push(target_piece);
                drop(downloaded);

                let string_path =
                    format!("{}/.tmp/{}", download.get_path(), target_piece.get_index());
                let download_file_path = Path::new(&string_path);
                if let Some(p) = download_file_path.parent() {
                    fs::create_dir_all(p).map_err(|err| err.to_string())?
                };

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(download_file_path)
                    .map_err(|err| err.to_string())?;
                file.write_all(&piece).map_err(|err| err.to_string())?;

                current_piece = None;
            }
            Ok(())
        });

        DownloadWorker {
            thread: Some(thread),
        }
    }

    pub fn get_thread(&mut self) -> Option<JoinHandle<Result<(), String>>> {
        self.thread.take()
    }
}

fn update_blacklist(
    blacklist_mutex: &Arc<Mutex<Vec<Peer>>>,
    peer: &Peer,
    peer_tx: &Sender<Peer>,
) -> Result<(), String> {
    let mut blacklist = blacklist_mutex
        .lock()
        .map_err(|_| "mutex lock error".to_string())?;

    blacklist.push(peer.clone());
    if blacklist.len() == blacklist.capacity() {
        let aux_blacklist = blacklist.clone();
        blacklist.clear();

        for blacklisted in aux_blacklist {
            peer_tx.send(blacklisted).map_err(|err| err.to_string())?;
        }
    }
    drop(blacklist);
    Ok(())
}
