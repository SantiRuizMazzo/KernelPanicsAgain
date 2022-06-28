use super::{peer::Peer, peer_protocol, torrent_piece::TorrentPiece};
use std::{
    fs::OpenOptions,
    io::Write,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct TorrentWorker {
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl TorrentWorker {
    pub fn new(
        peer: Peer,
        client_id: [u8; 20],
        info_hash: [u8; 20],
        sender: Sender<TorrentPiece>,
        receiver: Arc<Mutex<Receiver<TorrentPiece>>>,
    ) -> TorrentWorker {
        let thread = thread::spawn(move || {
            let target_piece = receiver
                .lock()
                .map_err(|_| "mutex lock error".to_string())?
                .recv()
                .map_err(|_| "channel receiver error".to_string())?;

            //A REVISAR!
            drop(receiver);

            println!("PIECE TO DOWNLOAD (INDEX): {}", target_piece.get_index());
            println!("PEER TO CONNECT: {:?}", peer);
            let piece =
                match peer_protocol::download_piece(peer, client_id, info_hash, target_piece) {
                    Ok(piece) => piece,
                    Err(error) => {
                        sender.send(target_piece).map_err(|err| err.to_string())?;
                        return Err(error);
                    }
                };

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(format!("{}", target_piece.get_index()))
                .map_err(|err| err.to_string())?;
            file.write_all(&piece).map_err(|err| err.to_string())
        });

        TorrentWorker {
            thread: Some(thread),
        }
    }

    pub fn get_thread(&mut self) -> Option<JoinHandle<Result<(), String>>> {
        self.thread.take()
    }
}
