use crate::{
    client::download::{peer::Peer, peer_protocol},
    logging::log_handle::LogHandle,
    messages::{
        message_types::{handshake::Handshake, have::Have, unchoke::Unchoke},
        peer_message::PeerMessage,
    },
    server::server_side::Notification,
};
use std::{
    collections::HashMap,
    io::{ErrorKind, Read},
    net::TcpStream,
    sync::{mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use super::upload_info::UploadInfo;

pub struct UploadWorker {
    id: usize,
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl UploadWorker {
    pub fn new(
        id: usize,
        mut stream: TcpStream,
        server_id: [u8; 20],
        torrents_mutex: Arc<Mutex<HashMap<[u8; 20], UploadInfo>>>,
        notif_tx: Sender<Notification>,
        log_handle: LogHandle,
    ) -> Result<Self, String> {
        let handshake = Self::validate_connection(&mut stream, server_id, torrents_mutex.clone())?;
        let mut peer = Self::connected_peer(&stream, handshake.peer_id())?;
        let info_hash = handshake.info_hash();

        let thread = thread::spawn(move || {
            let torrents = torrents_mutex.lock().map_err(|e| e.to_string())?;
            let upload_info = torrents.get(&info_hash).ok_or_else(|| {
                let _ = notif_tx.send(Notification::EndPeer(id));
                format!("Torrent {info_hash:?} is not served by this peer")
            })?;
            let mut local_bitfield = upload_info.bitfield().ok_or_else(|| {
                let _ = notif_tx.send(Notification::EndPeer(id));
                format!("Torrent {info_hash:?} does not have a bitfield")
            })?;
            let download_path = upload_info.download_path();
            drop(torrents);

            log_handle.log(&format!("Starting worker {id}"))?;

            local_bitfield.send(&mut stream).map_err(|e| {
                let _ = notif_tx.send(Notification::EndPeer(id));
                e.to_string()
            })?;
            //log_handle.log(&format!("> {local_bitfield:?}"))?;

            //Fix: every ?-handled error inside this loop may lead to end thread without sending a notification
            loop {
                if peer.is_interested() && peer.is_choked() {
                    let unchoke = Unchoke::new();
                    unchoke.send(&mut stream).map_err(|e| {
                        let _ = notif_tx.send(Notification::EndPeer(id));
                        e.to_string()
                    })?;
                    peer.unchoke();
                    //let _ = log_handle.log(&format!("> {unchoke:?}"));
                }

                let mut len = [0u8; 4];
                match stream.read(&mut len) {
                    Ok(n) => {
                        if n < len.len() {
                            let _ = log_handle.log("Read less bytes than len field has");
                            break;
                        }
                    }
                    Err(e) => {
                        //let _ = log_handle.log(&format!("Error reading len ({e:?})"));
                        if e.kind() != ErrorKind::WouldBlock {
                            break;
                        }
                        let torrents = torrents_mutex.lock().map_err(|e| e.to_string())?;
                        let upload_info = torrents.get(&info_hash).ok_or_else(|| {
                            let _ = notif_tx.send(Notification::EndPeer(id));
                            format!("Torrent {info_hash:?} is not served by this peer")
                        })?;
                        let updated_bitfield = upload_info.bitfield().ok_or_else(|| {
                            let _ = notif_tx.send(Notification::EndPeer(id));
                            format!("Torrent {info_hash:?} does not have a bitfield")
                        })?;

                        for piece_index in 0..local_bitfield.total_pieces() {
                            if (!local_bitfield.contains(piece_index))
                                && updated_bitfield.contains(piece_index)
                            {
                                let have = Have::new(piece_index as u32);
                                have.send(&mut stream).map_err(|e| {
                                    let _ = notif_tx.send(Notification::EndPeer(id));
                                    e.to_string()
                                })?;
                                //log_handle.log(&format!("> {have:?}"))?;
                            }
                        }

                        local_bitfield = updated_bitfield;
                        drop(torrents);
                        len = [0; 4];
                    }
                };

                let len = u32::from_be_bytes(len);
                let mut message_bytes = vec![0u8; len as usize];

                match stream.read(&mut message_bytes) {
                    Ok(n) => {
                        if n < message_bytes.len() {
                            let _ = log_handle.log(&format!(
                                "Read less bytes than message body has ({n}/{len})"
                            ));
                            continue;
                        }
                    }
                    Err(_e) => {
                        //let _ = log_handle.log(&format!("Error reading message body ({e:?})"));
                        break;
                    }
                };

                /*let message_bytes = match peer_protocol::read_message_bytes(&mut stream) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        let _ = log_handle.log(&format!("Error reading bytes from stream ({e})"));
                        break;
                    }
                };*/

                //log_handle.log(&format!("ID & PAYLOAD {message_bytes:?}"))?;
                let message = match PeerMessage::from(message_bytes) {
                    Ok(msg) => msg,
                    Err(e) => {
                        let _ = log_handle.log(&format!("Error parsing message ({e})"));
                        break;
                    }
                };

                //log_handle.log(&format!("< {message:?}"))?;
                match message {
                    PeerMessage::Interested => peer.set_interested(),
                    PeerMessage::NotInterested => peer.set_not_interested(),
                    PeerMessage::Request(request) => {
                        if peer_protocol::handle_request(
                            &mut stream,
                            request,
                            peer.is_choked(),
                            download_path.clone(),
                            &local_bitfield,
                            log_handle.clone(),
                        )
                        .is_err()
                        {
                            continue;
                        }
                    }
                    _ => continue,
                };

                let torrents = torrents_mutex.lock().map_err(|e| e.to_string())?;
                let upload_info = torrents.get(&info_hash).ok_or_else(|| {
                    let _ = notif_tx.send(Notification::EndPeer(id));
                    format!("Torrent {info_hash:?} is not served by this peer")
                })?;
                let updated_bitfield = upload_info.bitfield().ok_or_else(|| {
                    let _ = notif_tx.send(Notification::EndPeer(id));
                    format!("Torrent {info_hash:?} does not have a bitfield")
                })?;

                for piece_index in 0..local_bitfield.total_pieces() {
                    if (!local_bitfield.contains(piece_index))
                        && updated_bitfield.contains(piece_index)
                    {
                        let have = Have::new(piece_index as u32);
                        have.send(&mut stream).map_err(|e| {
                            let _ = notif_tx.send(Notification::EndPeer(id));
                            e.to_string()
                        })?;
                        //log_handle.log(&format!("> {have:?}"))?;
                    }
                }

                local_bitfield = updated_bitfield;
                drop(torrents);
            }

            let _ = notif_tx.send(Notification::EndPeer(id));
            Ok(())
        });

        Ok(Self {
            id,
            thread: Some(thread),
        })
    }

    fn validate_connection(
        stream: &mut TcpStream,
        server_id: [u8; 20],
        torrents_mutex: Arc<Mutex<HashMap<[u8; 20], UploadInfo>>>,
    ) -> Result<Handshake, String> {
        let handshake = peer_protocol::receive_handshake(stream).map_err(|e| e.to_string())?;

        let torrents = torrents_mutex.lock().map_err(|e| e.to_string())?;
        if !torrents.contains_key(&handshake.info_hash()) {
            return Err(format!("Torrent {:?} not served", handshake.info_hash()));
        }

        drop(torrents);
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .map_err(|e| e.to_string())?;
        peer_protocol::send_handshake(stream, server_id, handshake.info_hash())
            .map_err(|e| e.to_string())?;
        Ok(handshake)
    }

    fn connected_peer(stream: &TcpStream, peer_id: [u8; 20]) -> Result<Peer, String> {
        let peer_address = stream.peer_addr().map_err(|e| e.to_string())?;
        let ip = peer_address.ip().to_string();
        let port = peer_address.port();
        Ok(Peer::new(Some(peer_id), ip, port))
    }

    pub fn join(&mut self) -> Result<(), String> {
        self.thread
            .take()
            .ok_or(format!(
                "Error taking thread from upload worker {}",
                self.id
            ))?
            .join()
            .map_err(|_| format!("Error joining upload worker {}", self.id))?
    }

    pub fn id(&self) -> usize {
        self.id
    }
}
