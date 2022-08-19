use crate::{client::piece::Piece, config::Config, logging::log_handle::LogHandle};
use std::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::Receiver,
        mpsc::{SendError, Sender},
    },
    thread::{self, JoinHandle},
};

use super::upload::{upload_info::UploadInfo, upload_pool::UploadPool};

#[derive(Debug)]
pub enum Notification {
    NewPiece(Piece, UploadInfo),
    NewPeer(TcpStream),
    EndPeer(usize),
    EndServer,
}

pub struct ServerSide {
    id: [u8; 20],
    config: Config,
    log_handle: LogHandle,
}

impl ServerSide {
    pub fn new(id: [u8; 20], config: &Config, log_handle: LogHandle) -> Self {
        Self {
            id,
            config: config.clone(),
            log_handle,
        }
    }

    pub fn init(
        &mut self,
        notif_tx: Sender<Notification>,
        notif_rx: Receiver<Notification>,
    ) -> Result<(), String> {
        let _notification_thread = self.init_notifications(notif_tx.clone(), notif_rx)?;
        let _connection_thread = self.init_connections(notif_tx)?;
        Ok(())
        //Fix: notification and connection remain as zombie threads, must store their handles and join them
    }

    fn init_notifications(
        &self,
        notif_tx: Sender<Notification>,
        notif_rx: Receiver<Notification>,
    ) -> Result<JoinHandle<Result<(), String>>, String> {
        let log_handle = self.log_handle.clone();
        let mut pool = UploadPool::new(self.id);

        let thread: JoinHandle<Result<(), String>> = thread::spawn(move || {
            for notification in notif_rx {
                match notification {
                    Notification::NewPiece(piece, upload_info) => {
                        log_handle.log(&format!("Started serving piece {}", piece.index()))?;
                        pool.add_piece(piece, upload_info)?;
                    }
                    Notification::NewPeer(stream) => {
                        if let Err(e) = pool.add_worker(stream, &notif_tx, &log_handle) {
                            log_handle.log(&format!("Worker creation error: {e}"))?;
                        }
                    }
                    Notification::EndPeer(id) => {
                        pool.remove_worker(id)?;
                        log_handle.log(&format!("Removing worker {id}"))?;
                    }
                    Notification::EndServer => break,
                }
            }
            Ok(())
        });
        Ok(thread)
    }

    fn init_connections(
        &self,
        notif_tx: Sender<Notification>,
    ) -> Result<JoinHandle<Result<(), SendError<Notification>>>, String> {
        let socket = TcpListener::bind(self.config.server_address()).map_err(|e| e.to_string())?;

        //Fix: loop never ends, must implement a way to kill connection thread
        let thread: JoinHandle<Result<(), SendError<Notification>>> = thread::spawn(move || {
            for stream in socket.incoming().flatten() {
                notif_tx.send(Notification::NewPeer(stream))?
            }
            Ok(())
        });
        Ok(thread)
    }
}
