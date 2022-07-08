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
    peer_protocol::{self, DownloadError, BLOCK_SIZE},
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
        }
    }

    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn connect(&mut self, download: DownloadInfo) -> Result<TcpStream, DownloadError> {
        match TcpStream::connect(self.get_address()) {
            Ok(mut stream) => {
                peer_protocol::handle_handshake(&mut stream, download)
                    .map_err(DownloadError::Connection)?;
                Ok(stream)
            }
            Err(err) => Err(DownloadError::Connection(err.to_string())),
        }
    }

    pub fn download(
        &mut self,
        piece: TorrentPiece,
        connection: Option<TcpStream>,
        total_pieces: usize,
        download: DownloadInfo,
    ) -> Result<(TcpStream, Vec<u8>), DownloadError> {
        let mut cur_request = Request::new(piece.get_index() as u32, 0, BLOCK_SIZE);

        match connection {
            Some(mut stream) => {
                cur_request
                    .send(&mut stream)
                    .map_err(|err| DownloadError::Connection(err.to_string()))?;
                self.messages_loop(stream, cur_request, piece)
            }
            None => {
                let stream = self.connect(download)?;
                self.bitfield = vec![0; total_pieces];
                self.messages_loop(stream, cur_request, piece)
            }
        }
    }

    fn messages_loop(
        &mut self,
        mut stream: TcpStream,
        mut cur_request: Request,
        piece: TorrentPiece,
    ) -> Result<(TcpStream, Vec<u8>), DownloadError> {
        let mut downloaded = Vec::<u8>::with_capacity(piece.get_length());
        loop {
            let len = read_len(&mut stream)?;
            if len == 0 {
                continue;
            }

            let bytes_read = read_id_and_payload(&mut stream, len)?;
            let message = message_parser::parse(bytes_read).map_err(DownloadError::Piece)?;

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
                    piece.get_index(),
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
            }
        }
        Ok((stream, downloaded))
    }
}
