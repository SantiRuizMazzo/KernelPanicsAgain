use crate::{
    client::download::{
        peer::Peer,
        peer_protocol::{self, ProtocolError},
    },
    messages::{
        message_parser::{self, PeerMessage},
        message_type::{have::Have, unchoke::Unchoke},
    },
};
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle},
};

use super::{torrent_upload_info::UploadInfo, upload_pool::CleanerMessage};

pub struct UploadWorker {
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl UploadWorker {
    pub fn new(
        mut stream: TcpStream,
        mut peer: Peer,
        info_hash: [u8; 20],
        offered_torrents_mutex: Arc<Mutex<HashMap<[u8; 20], UploadInfo>>>,
        cleaner_tx: Sender<CleanerMessage>,
        hash_key: usize,
    ) -> UploadWorker {
        let thread = thread::spawn(move || {
            let offered_torrents = offered_torrents_mutex
                .lock()
                .map_err(|err| err.to_string())?;
            let upload_info = offered_torrents
                .get(&info_hash)
                .ok_or_else(|| "Torrent is not served by this peer".to_string())?;

            let mut local_bitfield = upload_info
                .get_bitfield()
                .ok_or_else(|| "Torrent is not served by this peer".to_string())?;
            let download_path = upload_info.get_path();
            drop(offered_torrents);

            local_bitfield
                .send(&mut stream)
                .map_err(|err| err.to_string())?;

            loop {
                if peer.is_interested() && peer.is_choked() {
                    Unchoke::new()
                        .send(&mut stream)
                        .map_err(|err| err.to_string())?;
                    peer.set_unchoked();
                }

                let len = match peer_protocol::read_len(&mut stream) {
                    Ok(len) => len,
                    Err(ProtocolError::Connection(_)) => {
                        break;
                    }
                    Err(_) => 0, // Skip if any other kind of error while reading len
                };

                if len == 0 {
                    continue;
                }

                let bytes_read = match peer_protocol::read_id_and_payload(&mut stream, len) {
                    Ok(bytes) => bytes,
                    Err(ProtocolError::Connection(_)) => {
                        break;
                    }
                    Err(_) => continue, // Skip if any other kind of error while reading len
                };

                let message = message_parser::parse(bytes_read).map_err(|err| err)?;

                match message {
                    PeerMessage::Interested(_) => {
                        peer.set_interested();
                    }
                    PeerMessage::NotInterested(_) => peer.set_not_interested(),
                    PeerMessage::Request(request) => peer_protocol::handle_request(
                        &mut stream,
                        request,
                        download_path.clone(),
                        &local_bitfield,
                    )?,
                    _ => {}
                };

                let offered_torrents = offered_torrents_mutex
                    .lock()
                    .map_err(|err| err.to_string())?;
                let upload_info = offered_torrents
                    .get(&info_hash)
                    .ok_or_else(|| "Torrent is not served by this peer".to_string())?;

                let updated_bitfield = upload_info
                    .get_bitfield()
                    .ok_or_else(|| "Torrent is not served by this peer".to_string())?;

                for index in 0..(local_bitfield.get_bits().len() * 8) {
                    if (!local_bitfield.contains(index)) && updated_bitfield.contains(index) {
                        Have::new(index as u32)
                            .send(&mut stream)
                            .map_err(|err| err.to_string())?;
                    }
                }

                local_bitfield = updated_bitfield;
                drop(offered_torrents);
            }
            let _ = cleaner_tx.send(CleanerMessage::RemoveWorker(hash_key));
            Ok(())
        });

        UploadWorker {
            thread: Some(thread),
        }
    }

    pub fn get_thread(&mut self) -> Option<JoinHandle<Result<(), String>>> {
        self.thread.take()
    }
}
