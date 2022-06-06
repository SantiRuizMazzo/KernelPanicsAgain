use crate::client::peer_protocol;

use super::{peer::Peer, torrent_piece::TorrentPiece};
use std::{
    fs::OpenOptions,
    io::Write,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct TorrentWorker {
    thread: Option<JoinHandle<()>>,
}

impl TorrentWorker {
    pub fn new(
        remote_peer: Peer,
        client_id: [u8; 20],
        info_hash: [u8; 20],
        receiver: Arc<Mutex<Receiver<TorrentPiece>>>,
    ) -> TorrentWorker {
        let thread = thread::spawn(move || loop {
            if let Ok(receiver_locked) = receiver.lock() {
                if let Ok(piece_to_download) = receiver_locked.recv() {
                    println!("> PIECE TO DOWNLOAD: {:?}", piece_to_download);
                    if let Err(msg) = peer_protocol::handle_communication(
                        remote_peer.clone(),
                        client_id,
                        info_hash,
                        piece_to_download,
                    ) {
                        if msg == "finished downloading piece" {
                            println!(" FINISHED ");
                            break;
                        }
                    }

                    if let Ok(downloaded_bytes) = peer_protocol::handle_communication(
                        remote_peer.clone(),
                        client_id,
                        info_hash,
                        piece_to_download,
                    ) {
                        if let Ok(mut file) = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(format!("{}", piece_to_download.get_index()))
                        {
                            let _ = file.write_all(&downloaded_bytes);
                        }
                    }
                }
            }
        });
        TorrentWorker {
            thread: Some(thread),
        }
    }

    pub fn get_thread(&mut self) -> Option<JoinHandle<()>> {
        self.thread.take()
    }
}
