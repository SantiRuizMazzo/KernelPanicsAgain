use super::{
    super::urlencoding::encode, single_file::SingleFile, torrent_decoding::torrent_from_bytes,
    tracker_decoding::tracker_info_from_bytes, tracker_info::TrackerInfo,
};
use native_tls::TlsConnector;
use std::{
    collections::HashMap,
    fs::read,
    io::{Error, ErrorKind, Read, Write},
    net::TcpStream,
    path::Path,
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

pub struct Torrent {
    pub announce: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
    pub files: Vec<SingleFile>,
    pub info_hash: [u8; 20],
    pub tracker_info: Option<TrackerInfo>,
}

impl Torrent {
    /// Builds a new `Torrent` instance with the given parameters.

    pub fn new(
        announce: String,
        piece_length: i64,
        pieces: Vec<u8>,
        files: Vec<SingleFile>,
        info_hash: [u8; 20],
    ) -> Torrent {
        Torrent {
            announce,
            piece_length,
            pieces,
            files,
            info_hash,
            tracker_info: None,
        }
    }

    /// Attempts to decode a .torrent file located at `path`, and build a `Torrent` struct with its data (if possible).

    pub fn from<P>(path: P) -> Result<Torrent, Error>
    where
        P: AsRef<Path>,
    {
        torrent_from_bytes(read(path)?)
    }

    /// Communicates with a tracker to request the list of peers which have the desired files at the moment.
    /// It returns a valid `TrackerInfo` struct with the data received from the tracker (if there wasn't
    /// any errors in the communication).

    pub fn get_tracker_info(&self, peer_id: [u8; 20], port: u32) -> Result<TrackerInfo, Error> {
        let tracker_addr = self.tracker_address()?;
        let query_dict = self.query_string_dict(peer_id, port)?;
        let tracker_req = self.tracker_request(&tracker_addr.domain, query_dict);
        let tracker_res = self.tracker_communication(tracker_addr, tracker_req)?;
        tracker_info_from_bytes(self.response_body(tracker_res)?)
    }

    /// Attempts to create a valid `ServerAddr` struct from the announce field of `Torrent`.

    fn tracker_address(&self) -> Result<ServerAddr, Error> {
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
    ) -> Result<HashMap<&str, String>, Error> {
        let mut query_dict = HashMap::new();
        query_dict.insert(
            "info_hash",
            encode(self.info_hash)
                .map_err(|_err| Error::new(ErrorKind::Other, "url encoding error"))?,
        );
        query_dict.insert(
            "peer_id",
            encode(peer_id).map_err(|_err| Error::new(ErrorKind::Other, "url encoding error"))?,
        );
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

    ///

    fn tracker_communication(
        &self,
        address: ServerAddr,
        request: String,
    ) -> Result<Vec<u8>, Error> {
        let mut stream = TcpStream::connect(address.dom_and_port())?;

        if address.protocol == "https" {
            let connector =
                TlsConnector::new().map_err(|_err| Error::new(ErrorKind::Other, "TLS error"))?;
            let mut stream = connector
                .connect(&address.domain, stream)
                .map_err(|_err| Error::new(ErrorKind::Other, "TLS error"))?;
            self.send_request(&request, &mut stream)
        } else {
            self.send_request(&request, &mut stream)
        }
    }

    ///

    fn send_request<T: Read + Write>(
        &self,
        request: &str,
        stream: &mut T,
    ) -> Result<Vec<u8>, Error> {
        stream.write_all(request.as_bytes())?;
        let mut buff = Vec::<u8>::new();
        stream.read_to_end(&mut buff)?;
        Ok(buff)
    }

    ///

    fn response_body(&self, response: Vec<u8>) -> Result<Vec<u8>, Error> {
        let body_start = match response.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            Some(headers_end) => headers_end + 4,
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "invalid tracker response formatting",
                ))
            }
        };
        Ok(response[body_start..response.len()].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sending_bytes_to_tracker() -> Result<(), Error> {
        let torrent = Torrent::from("tests/ubuntu.torrent")?;
        let peer_id = *b"01234567890123456789";
        let tracker_info = torrent.get_tracker_info(peer_id, 6881)?;
        println!("> TRACKER INFO FINAL:\n{:#?}", tracker_info);
        Ok(())
    }
}
