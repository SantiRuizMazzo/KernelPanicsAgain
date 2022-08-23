use crate::{
    client::{
        piece::Piece,
        single_file::SingleFile,
        torrent_decoding, tracker_decoding,
        tracker_info::{TrackerInfo, TrackerInfoState},
    },
    logging::log_handle::LogHandle,
    server::{server_side::Notification, upload::upload_info::UploadInfo},
    url_encoding, utils,
};
use native_tls::TlsConnector;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Error, Read, Write},
    net::TcpStream,
    path::Path,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    }, time::Instant,
};

use super::{
    client_side::DownloadedTorrents,
    download::{
        download_pool::{
            DownloadMessage, DownloadedPieces, PeerReceiver, PeerSender, PieceReceiver, PieceSender,
        },
        peer::Peer,
        peer_protocol::ProtocolError, download_worker_state::DownloadWorkerState,
    },
};

/// Represents a web server complete address.
#[derive(Debug)]
struct ServerAddr {
    protocol: String,
    domain: String,
    port: String,
}

impl ServerAddr {
    /// Creates an instance of `ServerAddr` from the given parameters.
    fn new(protocol: String, domain: String, port: String) -> Self {
        Self {
            protocol,
            domain,
            port,
        }
    }

    /// Returns the "domain:port" version of the `ServerAddr` as a string.
    fn dom_and_port(&self) -> String {
        format!("{}:{}", self.domain, self.port)
    }
}

/// Stores the information that a .torrent file contains.
#[derive(Debug, Clone)]
pub struct Torrent {
    name: String,
    announce: String,
    total_pieces: usize,
    files: Vec<SingleFile>,
    info_hash: [u8; 20],
    tracker_info: TrackerInfoState,
    piece_tx: PieceSender,
    piece_rx: PieceReceiver,
    peer_tx: PeerSender,
    peer_rx: PeerReceiver,
    downloaded: DownloadedPieces,
    download_path: String,
}

impl PartialEq for Torrent {
    fn eq(&self, other: &Self) -> bool {
        self.info_hash == other.info_hash
    }
}

impl Eq for Torrent {}

impl Torrent {
    /// Builds a new `Torrent` instance with the given parameters.
    pub fn new(
        name: String,
        announce: String,
        pieces: Vec<Piece>,
        files: Vec<SingleFile>,
        info_hash: [u8; 20],
    ) -> Result<Self, String> {
        let total_pieces = pieces.len();
        let (piece_tx, piece_rx) = Self::setup_pieces_queue(pieces)?;
        let (peer_tx, peer_rx) = Self::setup_peers_queue();
        let downloaded = Arc::new(Mutex::new(Vec::<Piece>::with_capacity(total_pieces)));

        Ok(Self {
            name,
            announce,
            total_pieces,
            files,
            info_hash,
            tracker_info: TrackerInfoState::Unset,
            piece_tx,
            piece_rx,
            peer_tx,
            peer_rx,
            downloaded,
            download_path: String::new(),
        })
    }

    pub fn get_total_size(&self) -> usize {
        self.files.iter().map(|f| f.length).sum::<i64>() as usize
    }

    fn setup_pieces_queue(pieces: Vec<Piece>) -> Result<(PieceSender, PieceReceiver), String> {
        let (piece_tx, piece_rx) = mpsc::channel::<DownloadMessage>();
        let pieces_queue = (piece_tx, Arc::new(Mutex::new(piece_rx)));

        for piece in pieces {
            pieces_queue
                .0
                .send(DownloadMessage::Piece(piece))
                .map_err(|e| e.to_string())?;
        }
        Ok(pieces_queue)
    }

    fn setup_peers_queue() -> (PeerSender, PeerReceiver) {
        let (peer_tx, peer_rx) = mpsc::channel::<Peer>();
        let peer_rx = Arc::new(Mutex::new(peer_rx));
        (peer_tx, peer_rx)
    }
    /// Attempts to decode a .torrent file located at `path`, and build a `Torrent` struct with its data (if possible).
    pub fn from<P>(path: P) -> Result<Self, String>
    where
        P: AsRef<Path>,
    {
        torrent_decoding::from_bytes(fs::read(path).map_err(|e| e.to_string())?)
    }

    /// Communicates with a tracker to request the list of peers which have the desired files at the moment.
    /// It returns a valid `TrackerInfo` struct with the data received from the tracker (if there wasn't
    /// any errors in the communication).
    pub fn request_tracker_info(
        &self,
        peer_id: [u8; 20],
        port: u32,
    ) -> Result<TrackerInfo, String> {
        let tracker_addr = self.tracker_address()?;
        let query_dict = self.query_string_dict(peer_id, port)?;
        let tracker_req = self.tracker_request(&tracker_addr.domain, query_dict);
        let tracker_res = self.tracker_communication(tracker_addr, tracker_req)?;
        tracker_decoding::from_bytes(self.response_body(tracker_res)?)
    }

    /// Attempts to create a valid `ServerAddr` struct from the announce field of `Torrent`.
    fn tracker_address(&self) -> Result<ServerAddr, String> {
        let tracker_addr = self.announce.replace("/announce", "");
        let mut tracker_addr = tracker_addr.split("://").collect::<Vec<&str>>();
        tracker_addr.append(&mut tracker_addr[1].split(':').collect::<Vec<&str>>());
        tracker_addr.remove(1);

        if tracker_addr.len() < 3 {
            if tracker_addr[0] == "https" {
                tracker_addr.push("443");
            } else {
                tracker_addr.push("80");
            }
        }
        Ok(ServerAddr::new(
            tracker_addr[0].to_string(),
            tracker_addr[1].to_string(),
            tracker_addr[2].to_string(),
        ))
    }

    /// Attempts to create a `HashMap` including every key-value pair that the query string of the tracker request must have.
    fn query_string_dict(
        &self,
        peer_id: [u8; 20],
        port: u32,
    ) -> Result<HashMap<&str, String>, String> {
        let mut query_dict = HashMap::new();
        query_dict.insert("info_hash", url_encoding::encode(self.info_hash)?);
        query_dict.insert("peer_id", url_encoding::encode(peer_id)?);
        query_dict.insert("port", port.to_string());
        query_dict.insert("uploaded", 0.to_string());
        query_dict.insert("downloaded", 0.to_string());

        let left = self.files.iter().fold(0, |acc, file| acc + file.length);
        query_dict.insert("left", left.to_string());
        query_dict.insert("event", "started".to_string());
        query_dict.insert("compact", "1".to_string());
        Ok(query_dict)
    }

    /// Given a `domain` and some `params`, it builds a string that holds a valid HTTP GET request, which is
    /// ready to be sent to the tracker.
    fn tracker_request(&self, domain: &str, params: HashMap<&str, String>) -> String {
        let mut http_get = String::from("GET /announce?");

        for (key, value) in params {
            http_get.push_str(&format!("{key}={value}&"));
        }

        http_get.pop();
        http_get.push_str(&format!(
            " HTTP/1.1\r\nHost: {domain}\r\nConnection: close\r\n\r\n"
        ));
        http_get
    }

    fn tracker_communication(
        &self,
        address: ServerAddr,
        request: String,
    ) -> Result<Vec<u8>, String> {
        let mut stream = TcpStream::connect(address.dom_and_port()).map_err(|e| e.to_string())?;

        if address.protocol == "https" {
            let connector = TlsConnector::new().map_err(|e| e.to_string())?;
            let mut stream = connector
                .connect(&address.domain, stream)
                .map_err(|e| e.to_string())?;
            self.send_request(&request, &mut stream)
        } else {
            self.send_request(&request, &mut stream)
        }
    }

    fn send_request<T: Read + Write>(
        &self,
        request: &str,
        stream: &mut T,
    ) -> Result<Vec<u8>, String> {
        let err = |e: Error| e.to_string();
        stream.write_all(request.as_bytes()).map_err(err)?;
        let mut buff = Vec::<u8>::new();
        stream.read_to_end(&mut buff).map_err(err)?;
        Ok(buff)
    }

    fn response_body(&self, response: Vec<u8>) -> Result<Vec<u8>, String> {
        if !response.starts_with(b"HTTP/1.1 200 OK\r\n") {
            return Err("Tracker did not accept request".to_string());
        }
        let body_start = match response.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            Some(headers_end) => headers_end + 4,
            None => return Err("invalid formatting of tracker response".to_string()),
        };
        Ok(response[body_start..response.len()].to_vec())
    }

    /// Gets peers list by sending a request to the tracker, and then it
    /// adds them to a peers queue
    pub fn load_peers(&mut self, client_id: [u8; 20]) -> Result<(), String> {
        if self.tracker_info.is_set() {
            return Ok(());
        }

        let tracker_info = self.request_tracker_info(client_id, 6881)?;
        //let tracker_info = TrackerInfo::new(0,vec![Peer::new(Some(client_id), "localhost".to_string(), 8081, 0)]);

        for peer in tracker_info.peers_list() {
            self.peer_tx.send(peer).map_err(|e| e.to_string())?;
        }

        self.tracker_info = TrackerInfoState::Set(tracker_info);
        Ok(())
    }

    pub fn build_files(&self) -> Result<(), String> {
        let mut current_piece_index = 0_usize;
        let mut piece_offset = 0;

        for file in &self.files {
            let mut opened_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}/{}", self.download_path, &file.path))
                .map_err(|e| e.to_string())?;

            let mut saved_file_length = 0_usize;
            let file_length = file.length as usize;

            while saved_file_length < file_length {
                let str_path = format!("{}/.tmp", self.download_path);
                let read_bytes = utils::read_piece_file(str_path, current_piece_index)?;
                let missing_bytes = file_length - saved_file_length;

                if (read_bytes.len() - piece_offset) > missing_bytes {
                    utils::append_to_file(
                        &mut opened_file,
                        read_bytes[piece_offset..(piece_offset + missing_bytes)].to_vec(),
                    )?;
                    saved_file_length += missing_bytes - piece_offset;
                    piece_offset += missing_bytes;
                } else {
                    utils::append_to_file(
                        &mut opened_file,
                        read_bytes[piece_offset..read_bytes.len()].to_vec(),
                    )?;
                    saved_file_length += read_bytes.len() - piece_offset;
                    piece_offset = 0;
                    current_piece_index += 1;
                }
            }
        }
        Ok(())
    }

    pub fn info_hash(&self) -> [u8; 20] {
        self.info_hash
    }

    pub fn total_pieces(&self) -> usize {
        self.total_pieces
    }

    pub fn save_in(&mut self, path: String) {
        self.download_path = format!("{}/{}", path, self.name)
    }

    pub fn download(
        &mut self,
        pieces_to_download: usize,
        client_id: [u8; 20],
        downloaded_torrents_mutex: DownloadedTorrents,
        notif_tx: Sender<Notification>,
        log_handle: &LogHandle,
        download_worker_id: usize,
        client_port: u32
    ) -> Result<(), String> {
        let (mut have_piece, mut have_peer) = (None, None);
        let mut download_counter = 0;

        while download_counter < pieces_to_download {
            if self.all_pieces_downloaded()? {
                return self.finish_download(downloaded_torrents_mutex);
            }

            let mut piece = self.get_new_piece(have_piece.take())?;
            let mut peer = self.get_new_peer(have_peer.take())?;
            let downloaded = self.downloaded.lock().map_err(|e| e.to_string())?;
            if (self.total_pieces - downloaded.len()) <= 20 {
                self.discard_piece(piece.clone())?
            }
            drop(downloaded);

            /* */

            match peer.download(&mut piece, self, client_id, log_handle) {
                Ok(()) => {
                    let last_download_time = Instant::now();
                    let final_path = format!("{}/.tmp/{}", self.download_path, piece.index());
                    if Path::new(&final_path).is_file() {
                        continue;
                    }

                    self.save_piece(&piece)?;
                    self.notify_piece(piece.clone(), notif_tx.clone())?;
                    self.update_status(piece, log_handle.clone())?;
                    download_counter += 1;
                    have_peer = Some(peer.clone());
                    let downloaded_pieces = self.downloaded.lock().map_err(|e| e.to_string())?;
                    let current_pieces = downloaded_pieces.len();

                    drop(downloaded_pieces);

                    let tracker_info = self.request_tracker_info(client_id, client_port)?;
                    let mut new_state = DownloadWorkerState::new(
                        download_worker_id,
                        self.info_hash,
                        peer.id(),
                        peer.ip(),
                        peer.port(),
                        self.name.clone(),
                        self.total_pieces,
                        Some(last_download_time),
                        tracker_info.peers_list().len()
                    );
                    new_state.total_size = self.get_total_size();
                    new_state.set_am_interested(peer.am_interested());
                    new_state.set_am_choked(peer.am_choked());
                    new_state.set_downloaded_pieces(current_pieces);
                    new_state.set_is_interested(peer.is_interested());
                    new_state.set_is_choked(peer.is_choked());

                    notif_tx
                        .send(Notification::UpdateUi(new_state))
                        .map_err(|err| err.to_string())?;
                }
                Err(ProtocolError::Piece(_)) => {
                    //let _ = log_handle.log(&format!("Changing piece {} -> {e}", piece.index()));
                    self.discard_piece(piece)?;
                    have_peer = Some(peer);
                }
                Err(ProtocolError::Peer(_)) => {
                    //let _ = log_handle.log(&format!("Changing peer {} -> {e}", peer.address()));
                    self.discard_peer(peer)?;
                    have_piece = Some(piece);
                }
            }
        }
        //Fix: when a download interval is finished peers and pieces may be lost due to not adding them to their queues again
        if let Some(piece) = have_piece {
            self.discard_piece(piece)?;
        }
        if let Some(peer) = have_peer {
            self.discard_peer(peer)?;
        }
        Ok(())
    }

    fn all_pieces_downloaded(&self) -> Result<bool, String> {
        let downloaded = self.downloaded.lock().map_err(|e| e.to_string())?;
        Ok(downloaded.len() == self.total_pieces)
    }

    fn finish_download(&self, downloaded_torrents_mutex: DownloadedTorrents) -> Result<(), String> {
        let mut downloaded_torrents = downloaded_torrents_mutex
            .lock()
            .map_err(|e| e.to_string())?;

        let am_downloaded = downloaded_torrents.iter().any(|torr| *torr == *self);
        if !am_downloaded {
            self.build_files()?;
            downloaded_torrents.push(self.clone());
        }
        Ok(())
    }

    fn get_new_piece(&self, have_piece: Option<Piece>) -> Result<Piece, String> {
        have_piece.map_or_else(
            || {
                let piece_rx = self.piece_rx.lock().map_err(|e| e.to_string())?;
                let message = piece_rx.recv().map_err(|e| e.to_string())?;
                drop(piece_rx);

                match message {
                    DownloadMessage::Piece(piece) => Ok(piece),
                    DownloadMessage::Kill => Err("Ending download".to_string()),
                }
            },
            Ok,
        )
    }

    fn get_new_peer(&self, have_peer: Option<Peer>) -> Result<Peer, String> {
        have_peer.map_or_else(
            || {
                let peer_rx = self.peer_rx.lock().map_err(|e| e.to_string())?;
                peer_rx.recv().map_err(|e| e.to_string())
            },
            Ok,
        )
    }

    fn save_piece(&self, piece: &Piece) -> Result<(), String> {
        let err = |e: Error| e.to_string();

        let final_path = format!("{}/.tmp/{}", self.download_path, piece.index());
        if let Some(path) = Path::new(&final_path).parent() {
            fs::create_dir_all(path).map_err(err)?
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(final_path)
            .map_err(err)?;
        file.write_all(&piece.bytes()).map_err(err)
    }

    fn notify_piece(&self, piece: Piece, notif_tx: Sender<Notification>) -> Result<(), String> {
        let upload = UploadInfo::new(
            self.info_hash,
            self.download_path.clone(),
            self.total_pieces,
        );

        notif_tx
            .send(Notification::NewPiece(piece, upload))
            .map_err(|e| e.to_string())
    }

    fn update_status(&self, piece: Piece, log_handle: LogHandle) -> Result<(), String> {
        let mut downloaded = self.downloaded.lock().map_err(|e| e.to_string())?;
        downloaded.push(piece.clone());

        let current_pieces = downloaded.len();
        let status = (current_pieces as f32 * 100_f32) / (self.total_pieces as f32);

        let msg = format!(
            "Downloaded piece {} from {} - Status: {current_pieces}/{} ({status:.2}%)",
            piece.index(),
            self.name,
            self.total_pieces
        );
        log_handle.log(&msg)
    }

    fn discard_piece(&self, piece: Piece) -> Result<(), String> {
        self.piece_tx
            .send(DownloadMessage::Piece(piece))
            .map_err(|e| e.to_string())
    }

    fn discard_peer(&self, peer: Peer) -> Result<(), String> {
        self.peer_tx.send(peer).map_err(|e| e.to_string())
    }

    pub fn downloaded(&self) -> DownloadedPieces {
        self.downloaded.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sending_bytes_to_tracker() -> Result<(), String> {
        let torrent = Torrent::from("tests/debian.torrent")?;
        let peer_id = *b"01234567890123456789";
        let tracker_info = torrent.request_tracker_info(peer_id, 6881)?;
        println!("> TRACKER INFO FINAL:\n{:#?}", tracker_info);
        Ok(())
    }
}
