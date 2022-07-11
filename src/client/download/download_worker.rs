use super::{download_info::DownloadInfo, peer::Peer};
use crate::{
    client::{
        client_side::{DownloadedTorrents, TorrentReceiver, TorrentSender},
        download::peer_protocol::ProtocolError,
        torrent_piece::TorrentPiece,
    },
    config::Config,
    logger::torrent_logger::LogMessage,
    server::{server_side::ServerNotification, upload::torrent_upload_info::UploadInfo},
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

pub enum DownloadMessage {
    Piece(TorrentPiece),
    Kill,
}

pub struct DownloadWorker {
    id: usize,
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl DownloadWorker {
    pub fn new(
        id: usize,
        torrent_queue: (TorrentSender, TorrentReceiver),
        downloaded_torrents_mutex: DownloadedTorrents,
        logger_tx: Sender<LogMessage>,
        client_id: [u8; 20],
        config: &Config,
        notification_tx: Sender<ServerNotification>,
    ) -> DownloadWorker {
        let download_path = config.get_download_path();
        let pieces_until_release = config.get_torrent_time_slice();

        let thread = thread::spawn(move || {
            let (torrent_tx, torrent_rx_mutex) = torrent_queue;

            loop {
                let torrent_rx = torrent_rx_mutex
                    .lock()
                    .map_err(|_| "mutex lock error".to_string())?;
                let mut torrent = torrent_rx.recv().map_err(|err| err.to_string())?;
                drop(torrent_rx);

                let downloaded_torrents = downloaded_torrents_mutex
                    .lock()
                    .map_err(|_| "mutex lock error".to_string())?;

                let already_downloaded = downloaded_torrents
                    .iter()
                    .any(|torr| torr.get_hash() == torrent.get_hash());

                if already_downloaded {
                    drop(downloaded_torrents);
                    continue;
                }

                drop(downloaded_torrents);

                torrent_tx
                    .send(torrent.clone())
                    .map_err(|err| err.to_string())?;

                //CAMBIAR NOMBRE DE START_DOWNLOAD
                torrent.start_download(client_id)?;

                let (piece_tx, piece_rx_mutex) = torrent.get_pieces_handle();
                let (peer_tx, peer_rx_mutex) = torrent.get_peers_handle();
                let downloaded_mutex = torrent.get_downloaded();

                let download_dir_path = format!("{download_path}/{}", torrent.get_name());
                let download = DownloadInfo::new(
                    client_id,
                    torrent.get_hash(),
                    download_dir_path,
                    torrent.get_total_pieces(),
                );

                let mut peer = Peer::new(Some([0; 20]), "".to_string(), 80, 0);
                let mut connection = None;
                let mut current_piece = None;
                let mut currently_downloaded = 0_usize;
                let mut piece;

                while currently_downloaded < pieces_until_release {
                    let downloaded = downloaded_mutex
                        .lock()
                        .map_err(|_| "mutex lock error".to_string())?;

                    let (mut current_pieces, mut total_pieces) =
                        (downloaded.len(), downloaded.capacity());

                    if current_pieces == total_pieces {
                        let mut downloaded_torrents = downloaded_torrents_mutex
                            .lock()
                            .map_err(|_| "mutex lock error".to_string())?;

                        let already_downloaded = downloaded_torrents
                            .iter()
                            .any(|torr| torr.get_hash() == torrent.get_hash());

                        if !already_downloaded {
                            downloaded_torrents.push(torrent);
                        }
                        drop(downloaded_torrents);
                        drop(downloaded);
                        break;
                    }

                    drop(downloaded);

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

                    (connection, piece) = match peer.download(
                        target_piece,
                        connection,
                        total_pieces,
                        download.clone(),
                        downloaded_mutex.clone(),
                    ) {
                        Ok((stream, piece)) => (Some(stream), piece),
                        Err(ProtocolError::Connection(_)) => {
                            peer_tx.send(peer.clone()).map_err(|err| err.to_string())?;
                            connection = None;
                            continue;
                        }
                        Err(ProtocolError::Piece(_)) => {
                            piece_tx
                                .send(DownloadMessage::Piece(target_piece))
                                .map_err(|err| err.to_string())?;
                            peer_tx.send(peer.clone()).map_err(|err| err.to_string())?;
                            current_piece = None;
                            connection = None;
                            continue;
                        }
                    };

                    store_piece(target_piece.get_index(), &piece, &download.get_path())?;

                    // Send have new piece to server
                    target_piece.notify_present(
                        notification_tx.clone(),
                        UploadInfo::new(
                            torrent.get_hash(),
                            download.get_path(),
                            torrent.get_total_pieces() as u32,
                        ),
                    )?;

                    let mut downloaded = downloaded_mutex
                        .lock()
                        .map_err(|_| "mutex lock error".to_string())?;

                    downloaded.push(target_piece);
                    (current_pieces, total_pieces) = (downloaded.len(), downloaded.capacity());
                    let progress = (current_pieces as f32 * 100_f32) / (total_pieces as f32);

                    logger_tx
                        .send(LogMessage::Log(format!(
                            "Downloaded piece {} from {} - Progress: {current_pieces}/{total_pieces} ({progress:.2}%)",
                            target_piece.get_index(), torrent.get_name()
                        )))
                        .map_err(|err| err.to_string())?;

                    drop(downloaded);
                    currently_downloaded += 1;
                    current_piece = None;
                }
            }

            //Ok(())
        });

        DownloadWorker {
            id,
            thread: Some(thread),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_thread(&mut self) -> Option<JoinHandle<Result<(), String>>> {
        self.thread.take()
    }
}

fn store_piece(piece_index: usize, piece_bytes: &[u8], download_path: &str) -> Result<(), String> {
    let final_path = format!("{download_path}/.tmp/{piece_index}");
    let final_path = Path::new(&final_path);
    if let Some(path) = final_path.parent() {
        fs::create_dir_all(path).map_err(|err| err.to_string())?
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(final_path)
        .map_err(|err| err.to_string())?;
    file.write_all(piece_bytes).map_err(|err| err.to_string())
}
