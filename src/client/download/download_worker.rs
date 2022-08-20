use crate::{
    client::client_side::{DownloadedTorrents, TorrentReceiver, TorrentSender},
    logging::log_handle::{LogHandle},
    server::server_side::Notification,
};
use std::{
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

pub struct DownloadWorker {
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl DownloadWorker {
    pub fn new(
        id: usize,
        client_id: [u8; 20],
        pieces_to_download: usize,
        torrent_tx: TorrentSender,
        torrent_rx_mutex: TorrentReceiver,
        downloaded_torrents_mutex: DownloadedTorrents,
        notif_tx: Sender<Notification>,
        log_handle: LogHandle,
        client_port: u32,
    ) -> Self {
        let thread = Some(thread::spawn(move || {
            //Fix: loop never ends, must implement a way to kill download workers
            loop {
                let torrent_rx = torrent_rx_mutex.lock().map_err(|e| e.to_string())?;
                let mut torrent = torrent_rx.recv().map_err(|e| e.to_string())?;
                drop(torrent_rx);

                let downloaded_torrents = downloaded_torrents_mutex
                    .lock()
                    .map_err(|e| e.to_string())?;

                if downloaded_torrents.iter().any(|torr| *torr == torrent) {
                    drop(downloaded_torrents);
                    continue;
                }

                drop(downloaded_torrents);

                torrent.load_peers(client_id)?;
                torrent_tx
                    .send(torrent.clone())
                    .map_err(|e| e.to_string())?;

                torrent.download(
                    pieces_to_download,
                    client_id,
                    downloaded_torrents_mutex.clone(),
                    notif_tx.clone(),
                    &log_handle,
                    id,
                    client_port.clone(),
                )?;
            }
        }));

        Self { thread }
    }

    pub fn join(&mut self) -> Result<(), String> {
        self.thread
            .take()
            .ok_or("Error taking thread from download worker")?
            .join()
            .map_err(|_| "Error joining download worker")?
    }
}
