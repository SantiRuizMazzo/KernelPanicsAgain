use crate::{
    client::{download::download_worker_state::DownloadWorkerState, piece::Piece},
    config::Config,
    logging::log_handle::LogHandle,
    ui_notification_structs::{peer_state::PeerState, ui_notification::UiNotification},
};
use gtk::glib::Sender as UiSender;
use std::{
    collections::HashMap,
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
    UpdateUi(DownloadWorkerState),
    EndServer,
}

pub struct ServerSide {
    id: [u8; 20],
    config: Config,
    log_handle: LogHandle,
    ui_sender: Option<UiSender<UiNotification>>,
}

impl ServerSide {
    pub fn new(id: [u8; 20], config: &Config, log_handle: LogHandle) -> Self {
        Self {
            id,
            config: config.clone(),
            log_handle,
            ui_sender: None,
        }
    }

    pub fn set_ui_sender(&mut self, sender: Option<UiSender<UiNotification>>) {
        self.ui_sender = sender
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
        let mut hash_states: HashMap<usize, DownloadWorkerState> =
            HashMap::<usize, DownloadWorkerState>::new();
        let ui_option = self.ui_sender.clone();

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
                    Notification::UpdateUi(sendable) => {
                        if ui_option.is_some() {
                            hash_states.insert(sendable.id, sendable);
                            let ui_notif =
                                ServerSide::generate_ui_notification(hash_states.clone());
                            let clone_ui_option = ui_option.clone();
                            let _ = match clone_ui_option {
                                Some(sender) => sender.send(ui_notif),
                                None => Ok(()),
                            };
                        }
                    }
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

    pub fn set_peer_id(&mut self, peer_id: [u8; 20]) {
        self.id = peer_id;
    }
    pub fn get_different_peers(hash_map: HashMap<usize, DownloadWorkerState>) -> usize {
        let mut hash_count = HashMap::<[u8; 20], bool>::new();

        for download_worker_state in hash_map {
            if let Some(peer_id) = download_worker_state.1.curr_peer_id {
                hash_count.insert(peer_id, true);
            }
        }
        hash_count.len()
    }
    pub fn generate_ui_notification(
        hash_to_send: HashMap<usize, DownloadWorkerState>,
    ) -> UiNotification {
        let mut ui_notif = UiNotification::new();
        for download_worker_state in hash_to_send.iter() {
            let different_peers = ServerSide::get_different_peers(hash_to_send.clone());
            let peer_states = ServerSide::get_peer_states_vec(hash_to_send.clone());
            let torrent_state = download_worker_state
                .1
                .generate_torrent_state(different_peers, peer_states);
            ui_notif.add_torrent_state(torrent_state);
        }
        ui_notif
    }
    pub fn get_peer_states_vec(
        hash_to_send: HashMap<usize, DownloadWorkerState>,
    ) -> Vec<PeerState> {
        let mut res = Vec::<PeerState>::new();
        for (_id, state) in hash_to_send {
            res.push(state.generate_peer_state());
        }
        res
    }
}
