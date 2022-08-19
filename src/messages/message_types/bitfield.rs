use crate::{client::download::peer_protocol::ProtocolError, utils};
use std::{
    io::{Error, Write},
    net::TcpStream,
};

pub const BITFIELD_ID: u8 = 5;
const BITS_IN_BYTE: usize = 8;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Bitfield {
    len: u32,
    id: u8,
    bitfield: Vec<u8>,
}

impl Default for Bitfield {
    fn default() -> Self {
        Self {
            len: 1,
            id: BITFIELD_ID,
            bitfield: Vec::new(),
        }
    }
}

impl Bitfield {
    pub fn new(bitfield: Vec<u8>) -> Self {
        Self {
            len: bitfield.len() as u32 + 1,
            id: BITFIELD_ID,
            bitfield,
        }
    }

    pub fn from(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        Ok(Self::new(Vec::from(&bytes[1..])))
    }

    pub fn bits(&self) -> Vec<u8> {
        self.bitfield.clone()
    }

    pub fn add_piece(&mut self, piece_index: usize) {
        self.bitfield[piece_index / BITS_IN_BYTE] |= self.piece_bit_mask(piece_index)
    }

    pub fn set_size(&mut self, total_pieces: usize) {
        let total_bytes = (total_pieces + BITS_IN_BYTE - 1) / BITS_IN_BYTE;
        self.bitfield = vec![0; total_bytes];
    }

    pub fn contains(&self, piece_index: usize) -> bool {
        let byte = self.bitfield[piece_index / BITS_IN_BYTE];
        let mask = self.piece_bit_mask(piece_index);
        (byte & mask) != 0
    }

    fn piece_bit_mask(&self, piece_index: usize) -> u8 {
        let byte_end = utils::round_up(piece_index, BITS_IN_BYTE);
        let mut shift = 7;
        if byte_end > piece_index {
            shift = byte_end - 1 - piece_index;
        }
        1 << shift
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
        stream.write_all(&self.len.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.id.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.bitfield).map_err(err)
    }

    pub fn total_pieces(&self) -> usize {
        self.bitfield.len() * BITS_IN_BYTE
    }
}
