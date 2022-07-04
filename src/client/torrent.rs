use crate::{
    client::{
        download::download_pool::DownloadPool,
        single_file::SingleFile,
        torrent_decoding,
        torrent_piece::TorrentPiece,
        tracker_decoding,
        tracker_info::{TrackerInfo, TrackerInfoState},
    },
    urlencoding,
    utils::{append_to_file, read_piece_file},
};
use native_tls::TlsConnector;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

const TOTAL_WORKERS: usize = 20;

/// Represents a web server complete address.
#[derive(Debug)]
struct ServerAddr {
    protocol: String,
    domain: String,
    port: String,
}

impl ServerAddr {
    /// Creates an instance of `ServerAddr` from the given parameters.
    fn new(protocol: String, domain: String, port: String) -> ServerAddr {
        ServerAddr {
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
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Torrent {
    announce: String,
    piece_length: usize,
    pieces: Vec<TorrentPiece>,
    files: Vec<SingleFile>,
    info_hash: [u8; 20],
    tracker_info: TrackerInfoState,
    index: usize,
}

impl Torrent {
    /// Builds a new `Torrent` instance with the given parameters.
    pub fn new(
        announce: String,
        piece_length: usize,
        pieces: Vec<TorrentPiece>,
        files: Vec<SingleFile>,
        info_hash: [u8; 20],
        index: usize,
    ) -> Torrent {
        Torrent {
            announce,
            piece_length,
            pieces,
            files,
            info_hash,
            tracker_info: TrackerInfoState::Unset,
            index,
        }
    }
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
        for piece in self.pieces.iter_mut() {
            piece.set_torrent_index(index);
        }
    }
    /// Attempts to decode a .torrent file located at `path`, and build a `Torrent` struct with its data (if possible).
    pub fn from<P>(path: P) -> Result<Torrent, String>
    where
        P: AsRef<Path>,
    {
        torrent_decoding::torrent_from_bytes(fs::read(path).map_err(|err| err.to_string())?)
    }

    /// Communicates with a tracker to request the list of peers which have the desired files at the moment.
    /// It returns a valid `TrackerInfo` struct with the data received from the tracker (if there wasn't
    /// any errors in the communication).
    pub fn get_tracker_info(&self, peer_id: [u8; 20], port: u32) -> Result<TrackerInfo, String> {
        let tracker_addr = self.tracker_address()?;
        let query_dict = self.query_string_dict(peer_id, port)?;
        let tracker_req = self.tracker_request(&tracker_addr.domain, query_dict);
        let tracker_res = self.tracker_communication(tracker_addr, tracker_req)?;
        //println!("TRACKER RESPONSE:\n{}", String::from_utf8_lossy(&tracker_res));
        tracker_decoding::tracker_info_from_bytes(self.response_body(tracker_res)?)
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
        query_dict.insert("info_hash", urlencoding::encode(self.info_hash)?);
        query_dict.insert("peer_id", urlencoding::encode(peer_id)?);
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
            http_get.push_str(&format!("{}={}&", key, value));
        }

        http_get.pop();
        http_get.push_str(&format!(
            " HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            domain
        ));
        http_get
    }

    fn tracker_communication(
        &self,
        address: ServerAddr,
        request: String,
    ) -> Result<Vec<u8>, String> {
        let mut stream =
            TcpStream::connect(address.dom_and_port()).map_err(|err| err.to_string())?;

        if address.protocol == "https" {
            let connector = TlsConnector::new().map_err(|err| err.to_string())?;
            let mut stream = connector
                .connect(&address.domain, stream)
                .map_err(|err| err.to_string())?;
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
        stream
            .write_all(request.as_bytes())
            .map_err(|err| err.to_string())?;
        let mut buff = Vec::<u8>::new();
        stream
            .read_to_end(&mut buff)
            .map_err(|err| err.to_string())?;
        Ok(buff)
    }

    fn response_body(&self, response: Vec<u8>) -> Result<Vec<u8>, String> {
        if !response.starts_with(b"HTTP/1.1 200 OK\r\n") {
            return Err("tracker request sent, not accepted by tracker".to_string());
        }
        let body_start = match response.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            Some(headers_end) => headers_end + 4,
            None => return Err("invalid formatting of tracker response".to_string()),
        };
        Ok(response[body_start..response.len()].to_vec())
    }

    /// Starts torrent download by requesting tracker info and connecting to peers
    pub fn start_download(
        &mut self,
        client_id: [u8; 20],
        torrent_index: usize,
    ) -> Result<(), String> {
        self.tracker_info = TrackerInfoState::Set(self.get_tracker_info(client_id, 6881)?);
        let peers = self
            .tracker_info
            .get_peers()
            .ok_or_else(|| "no peer list loaded".to_string())?;

        println!("TOTAL PIECES {}", self.pieces.len());
        println!("TOTAL PEERS {}", peers.len());
        //self.pieces = vec![self.pieces.remove(self.pieces.len() - 1)];
        let pool = DownloadPool::new(
            TOTAL_WORKERS,
            &self.pieces,
            &peers,
            client_id,
            self.info_hash,
        )?;
        drop(pool);
        self.create_downloaded_files(torrent_index)?;

        Ok(())
    }

    pub fn create_downloaded_files(&self, torrent_index: usize) -> Result<(), String> {
        let mut current_piece_index = 0_usize;
        let mut piece_offset = 0;

        for file in &self.files {
            let mut opened_file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(format!(
                    "downloads/tmp{}/{}",
                    torrent_index,
                    file.path.clone()
                ))
                .map_err(|err| err.to_string())?;

            println!("{opened_file:?}");
            let mut saved_file_length = 0_usize;
            let file_length = file.length as usize;

            while saved_file_length < file_length {
                let str_path = format!("downloads/tmp{}/", torrent_index);
                let read_bytes = read_piece_file(str_path, current_piece_index)?;
                let missing_bytes = file_length - saved_file_length;

                if (read_bytes.len() - piece_offset) > missing_bytes {
                    append_to_file(
                        &mut opened_file,
                        read_bytes[piece_offset..(piece_offset + missing_bytes)].to_vec(),
                    )?;
                    saved_file_length += missing_bytes - piece_offset;
                    piece_offset += missing_bytes;
                } else {
                    append_to_file(
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sending_bytes_to_tracker() -> Result<(), String> {
        let torrent = Torrent::from("tests/debian.torrent")?;
        let peer_id = *b"01234567890123456789";
        let tracker_info = torrent.get_tracker_info(peer_id, 6881)?;
        println!("> TRACKER INFO FINAL:\n{:#?}", tracker_info);
        Ok(())
    }
}
