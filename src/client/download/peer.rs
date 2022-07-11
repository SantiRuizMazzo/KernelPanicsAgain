use crate::{
    client::download::peer_protocol::{
        handle_bitfield, handle_choke, handle_have, handle_piece, handle_unchoke,
        read_id_and_payload, read_len,
    },
    messages::{
        message_parser::{self, PeerMessage},
        message_type::request::Request,
    },
};

use super::{
    super::torrent_piece::TorrentPiece,
    download_info::DownloadInfo,
    download_pool::DownloadedPieces,
    peer_protocol::{self, ProtocolError, BLOCK_SIZE},
};
use std::net::TcpStream;

/// Stores information about each peer in the peer list that is provided by the tracker.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Peer {
    id: Option<[u8; 20]>,
    ip: String,
    port: u32,
    index: usize,
    bitfield: Vec<u8>,
    am_interested: bool,
    am_choked: bool,
    is_interested: bool,
    is_choked: bool,
}

impl Peer {
    pub fn new(id: Option<[u8; 20]>, ip: String, port: u32, index: usize) -> Peer {
        Peer {
            id,
            ip,
            port,
            index,
            bitfield: Vec::new(),
            am_interested: false,
            am_choked: true,
            is_interested: false,
            is_choked: true,
        }
    }
    pub fn get_id(&self) -> Option<[u8; 20]> {
        self.id
    }
    pub fn get_ip(&self) -> String {
        self.ip.clone()
    }
    pub fn get_port(&self) -> u32 {
        self.port
    }
    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn connect(&mut self, download: DownloadInfo) -> Result<TcpStream, ProtocolError> {
        match TcpStream::connect(self.get_address()) {
            Ok(mut stream) => {
                peer_protocol::handle_handshake(&mut stream, download)
                    .map_err(ProtocolError::Connection)?;
                Ok(stream)
            }
            Err(err) => Err(ProtocolError::Connection(err.to_string())),
        }
    }

    pub fn download(
        &mut self,
        piece: TorrentPiece,
        connection: Option<TcpStream>,
        total_pieces: usize,
        download: DownloadInfo,
        downloaded_mutex: DownloadedPieces,
    ) -> Result<(TcpStream, Vec<u8>), ProtocolError> {
        let mut cur_request = Request::new(piece.get_index() as u32, 0, BLOCK_SIZE);

        match connection {
            Some(mut stream) => {
                cur_request
                    .send(&mut stream)
                    .map_err(|err| ProtocolError::Connection(err.to_string()))?;
                self.download_messages_loop(stream, cur_request, piece, downloaded_mutex)
            }
            None => {
                let stream = self.connect(download)?;
                self.bitfield = vec![0; total_pieces];
                self.download_messages_loop(stream, cur_request, piece, downloaded_mutex)
            }
        }
    }

    fn download_messages_loop(
        &mut self,
        mut stream: TcpStream,
        mut cur_request: Request,
        piece: TorrentPiece,
        downloaded_mutex: DownloadedPieces,
    ) -> Result<(TcpStream, Vec<u8>), ProtocolError> {
        let mut downloaded = Vec::<u8>::with_capacity(piece.get_length());
        loop {
            let len = read_len(&mut stream)?;
            if len == 0 {
                continue;
            }

            let bytes_read = read_id_and_payload(&mut stream, len)?;
            let message = message_parser::parse(bytes_read).map_err(ProtocolError::Piece)?;

            match message {
                PeerMessage::Bitfield(msg) => {
                    self.bitfield = handle_bitfield(
                        &mut stream,
                        msg.get_bits(),
                        piece.get_index(),
                        &mut self.am_interested,
                    )?;
                }
                PeerMessage::Have(msg) => handle_have(
                    &mut stream,
                    msg,
                    &mut self.bitfield,
                    &mut self.am_interested,
                    downloaded_mutex.clone(),
                )?,
                PeerMessage::Unchoke(_) => handle_unchoke(
                    &mut stream,
                    &mut cur_request,
                    &mut self.am_choked,
                    self.am_interested,
                )?,
                PeerMessage::Choke(_) => handle_choke(&mut cur_request, &mut self.am_choked),
                PeerMessage::Piece(msg) => {
                    let bytes_downloaded = handle_piece(
                        &mut stream,
                        msg,
                        &mut downloaded,
                        piece,
                        &mut cur_request,
                        self.am_choked,
                    )?;
                    if bytes_downloaded == piece.get_length() {
                        break;
                    }
                }
                PeerMessage::Cancel(msg) => {
                    return Err(ProtocolError::Piece(format!(
                        "canceled piece {} beginning at {} request",
                        msg.get_index(),
                        msg.get_begin()
                    )))
                }
                _ => {}
            }
        }
        Ok((stream, downloaded))
    }

    pub fn is_interested(&self) -> bool {
        self.is_interested
    }

    pub fn set_interested(&mut self) {
        self.is_interested = true;
    }

    pub fn set_not_interested(&mut self) {
        self.is_interested = false;
    }

    pub fn is_choked(&self) -> bool {
        self.is_choked
    }

    pub fn set_unchoked(&mut self) {
        self.is_choked = false
    }
}
