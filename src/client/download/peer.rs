use crate::{
    client::{piece::Piece, torrent::Torrent},
    logging::log_handle::LogHandle,
    messages::{
        message_types::{bitfield::Bitfield, interested::Interested, request::Request},
        peer_message::PeerMessage,
    },
};

use super::{
    download_pool::DownloadedPieces,
    peer_protocol::{self, ProtocolError},
};
use std::{net::TcpStream, time::Duration};

/// Stores information about each peer in the peer list that is provided by the tracker.
#[derive(Debug)]
pub struct Peer {
    id: Option<[u8; 20]>,
    ip: String,
    port: u16,
    bitfield: Bitfield,
    am_interested: bool,
    am_choked: bool,
    is_interested: bool,
    is_choked: bool,
    connection: Option<TcpStream>,
    last_request: Request,
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.ip == other.ip
            && self.port == other.port
            && self.bitfield == other.bitfield
            && self.am_interested == other.am_interested
            && self.am_choked == other.am_choked
            && self.is_interested == other.is_interested
            && self.is_choked == other.is_choked
    }
}

impl Eq for Peer {}

impl Clone for Peer {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            ip: self.ip.clone(),
            port: self.port,
            bitfield: self.bitfield.clone(),
            am_interested: self.am_interested,
            am_choked: self.am_choked,
            is_interested: self.is_interested,
            is_choked: self.is_choked,
            connection: None,
            last_request: Request::default(),
        }
    }
}

impl Default for Peer {
    fn default() -> Self {
        Self {
            id: None,
            ip: "localhost".to_string(),
            port: 6881,
            bitfield: Bitfield::default(),
            am_interested: false,
            am_choked: true,
            is_interested: false,
            is_choked: true,
            connection: None,
            last_request: Request::default(),
        }
    }
}

impl Peer {
    pub fn new(id: Option<[u8; 20]>, ip: String, port: u16) -> Self {
        Self {
            id,
            ip,
            port,
            bitfield: Bitfield::default(),
            am_interested: false,
            am_choked: true,
            is_interested: false,
            is_choked: true,
            connection: None,
            last_request: Request::default(),
        }
    }

    pub fn id(&self) -> Option<[u8; 20]> {
        self.id
    }

    pub fn ip(&self) -> String {
        self.ip.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    fn open_connection(
        &mut self,
        client_id: [u8; 20],
        torrent: &Torrent,
        log_handle: LogHandle,
    ) -> Result<TcpStream, ProtocolError> {
        /*log_handle
        .log(&format!("Connecting to: {}", self.address()))
        .map_err(ProtocolError::Peer)?;*/
        let mut stream =
            TcpStream::connect(self.address()).map_err(|e| ProtocolError::Peer(e.to_string()))?;
        /*log_handle
        .log(&format!("Connected to: {}", self.address()))
        .map_err(ProtocolError::Peer)?;*/
        peer_protocol::handle_handshakes(&mut stream, client_id, torrent.info_hash())?;
        log_handle
            .log(&format!("Handshaked with: {}", self.address()))
            .map_err(ProtocolError::Peer)?;
        self.bitfield.set_size(torrent.total_pieces());
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| ProtocolError::Peer(e.to_string()))?;
        Ok(stream)
    }

    fn reuse_connection(
        &mut self,
        mut stream: TcpStream,
        piece_index: usize,
        _log_handle: LogHandle,
    ) -> Result<TcpStream, ProtocolError> {
        if self.bitfield.contains(piece_index) {
            if !self.am_interested {
                let interested = Interested::new();
                interested.send(&mut stream)?;
                self.am_interested = true;
                //let _ = log_handle.log(&format!("> {interested:?}"));
            }
            if !self.am_choked && self.am_interested {
                self.last_request.send(&mut stream)?;
                //let _ = log_handle.log(&format!("> {:?}", self.last_request));
            }
        }
        Ok(stream)
    }

    pub fn download(
        &mut self,
        piece: &mut Piece,
        torrent: &Torrent,
        client_id: [u8; 20],
        log_handle: &LogHandle,
    ) -> Result<(), ProtocolError> {
        self.last_request = piece.request_next_block();

        let mut stream = match self.connection.take() {
            None => self.open_connection(client_id, torrent, log_handle.clone())?,
            Some(stream) => self.reuse_connection(stream, piece.index(), log_handle.clone())?,
        };

        match self.handle_messages(&mut stream, piece, torrent.downloaded(), log_handle) {
            Ok(()) => {
                self.connection = Some(stream);
                Ok(())
            }
            Err(ProtocolError::Piece(e)) => {
                self.connection = Some(stream);
                Err(ProtocolError::Piece(e))
            }
            Err(ProtocolError::Peer(e)) => Err(ProtocolError::Peer(e)),
        }
    }

    fn handle_messages(
        &mut self,
        stream: &mut TcpStream,
        piece: &mut Piece,
        downloaded_mutex: DownloadedPieces,
        log_handle: &LogHandle,
    ) -> Result<(), ProtocolError> {
        loop {
            let message_bytes = peer_protocol::read_message_bytes(stream)?;
            let message = PeerMessage::from(message_bytes)?;
            //let _ = log_handle.log(&format!("< {message:?}"));

            match message {
                PeerMessage::Choke => {
                    peer_protocol::handle_choke(&mut self.last_request, &mut self.am_choked)
                }
                PeerMessage::Unchoke => peer_protocol::handle_unchoke(
                    stream,
                    &mut self.last_request,
                    &mut self.am_choked,
                    self.am_interested,
                    log_handle.clone(),
                )?,
                PeerMessage::Have(have) => peer_protocol::handle_have(
                    stream,
                    have,
                    &mut self.bitfield,
                    &mut self.am_interested,
                    downloaded_mutex.clone(),
                    log_handle.clone(),
                )?,
                PeerMessage::Bitfield(bitfield) => {
                    self.bitfield = bitfield;
                    peer_protocol::handle_bitfield(
                        stream,
                        &mut self.bitfield,
                        piece.index(),
                        &mut self.am_interested,
                        log_handle.clone(),
                    )?;
                }
                PeerMessage::Block(block) => {
                    peer_protocol::handle_block(
                        stream,
                        block,
                        piece,
                        &mut self.last_request,
                        self.am_choked,
                        log_handle.clone(),
                    )?;

                    if piece.is_full() {
                        break;
                    }
                }
                PeerMessage::Cancel(cancel) => {
                    return Err(ProtocolError::Piece(format!(
                        "Canceled request of piece {} beginning at {}",
                        cancel.index(),
                        cancel.begin()
                    )))
                }
                _ => continue,
            }
        }
        Ok(())
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

    pub fn unchoke(&mut self) {
        self.is_choked = false
    }
}
