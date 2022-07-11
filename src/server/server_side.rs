use crate::{
    client::{download::peer_protocol, torrent_piece::TorrentPiece},
    config::Config,
    logger::torrent_logger::LogMessage,
    messages::message_type::handshake::HandShake,
};
use std::{
    net::{Shutdown, TcpListener, TcpStream},
    sync::{mpsc::Receiver, mpsc::Sender},
    thread::{self, JoinHandle},
};

use super::upload::{torrent_upload_info::UploadInfo, upload_pool::UploadPool};

#[derive(Debug)]
pub enum ServerNotification {
    NewConnection(TcpStream, HandShake),
    NewPiece(TorrentPiece, UploadInfo),
    Kill,
}

pub struct ServerSide {
    config: Config,
    peer_id: [u8; 20],
    log_sender: Sender<LogMessage>,
}

impl ServerSide {
    pub fn new(config: Config, log_sender: Sender<LogMessage>) -> ServerSide {
        ServerSide {
            config,
            peer_id: [0; 20],
            log_sender,
        }
    }

    pub fn init(
        &mut self,
        notification_sender: Sender<ServerNotification>,
        notification_receiver: Receiver<ServerNotification>,
    ) -> Result<(), String> {
        let client_id = self.peer_id;
        let log_tx = self.log_sender.clone();

        let _notification_thread: JoinHandle<Result<(), String>> = thread::spawn(move || {
            let mut upload_pool = UploadPool::new();

            loop {
                let notification = notification_receiver
                    .recv()
                    .map_err(|_| "Error while reading from notification channel".to_string())?;

                match notification {
                    ServerNotification::NewConnection(mut stream, received_handshake) => {
                        let _ = log_tx.send(LogMessage::Log(format!(
                            "New connection detected! {:?}",
                            received_handshake
                        )));

                        let is_serving =
                            upload_pool.is_serving(received_handshake.get_info_hash())?;
                        if !is_serving {
                            let _ = stream.shutdown(Shutdown::Both);
                            let _ = log_tx.send(LogMessage::Log(format!(
                                "Torrent not served, Shuting down connection! {:?}",
                                received_handshake
                            )));
                        } else {
                            let handshake = HandShake::new(
                                "BitTorrent protocol".to_string(),
                                [0; 8],
                                received_handshake.get_info_hash(),
                                client_id,
                            );
                            let _ = handshake.send(&mut stream);

                            if let Err(err) = handshake.send(&mut stream) {
                                let _ = log_tx.send(LogMessage::Log(format!(
                                    "Error {err} sending handshake {:?}",
                                    handshake
                                )));
                                let _ = stream.shutdown(Shutdown::Both);
                                continue;
                            }

                            if let Err(err) =
                                upload_pool.add_new_connection(stream, received_handshake)
                            {
                                let _ = log_tx.send(LogMessage::Log(format!(
                                    "Error {err} creating new worker",
                                )));
                                continue;
                            }

                            let _ = log_tx.send(LogMessage::Log(format!(
                                "New connection established! {:?}",
                                handshake
                            )));
                        }
                    }
                    ServerNotification::NewPiece(torrent_piece, torrent_upload_info) => {
                        // Update current bitfield for torrent
                        upload_pool.offer_new_piece(torrent_piece, torrent_upload_info)?;
                        // Send
                    }
                    ServerNotification::Kill => break,
                }
            }
            Ok(())
        });

        let sender_connections = notification_sender;
        let listener =
            TcpListener::bind(self.config.get_server_address()).map_err(|err| err.to_string())?;

        let _connection_thread = thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        if let Ok(handshake) = peer_protocol::handle_hs_receiving(&mut stream) {
                            let _ = sender_connections
                                .send(ServerNotification::NewConnection(stream, handshake));
                        }
                    }
                    Err(e) => {
                        println!("Connection fail {:?}", e);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn set_peer_id(&mut self, peer_id: [u8; 20]) {
        self.peer_id = peer_id;
    }
}
