use std::io::Write;
use std::net::TcpStream;

use crate::utils;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Bitfield {
    // 4 byte
    len: u32,
    // 1 byte
    id: u8,
    // uso un vec en lugar del [a,b] que veniamos usando porque no puedo saber el largo del
    //bitfield en tiempo de compilacion
    bitfield: Vec<u8>,
}

impl Bitfield {
    pub fn new(bitfield: Vec<u8>) -> Bitfield {
        Bitfield {
            len: bitfield.len() as u32 + 1,
            id: 5,
            bitfield,
        }
    }

    pub fn get_bits(&self) -> Vec<u8> {
        self.bitfield.clone()
    }
    pub fn set_have_piece(&mut self, piece_index: usize) {
        self.bitfield[piece_index / 8] |= self.piece_bit_mask(piece_index);
    }

    fn piece_bit_mask(&self, piece_index: usize) -> u8 {
        let byte_end = utils::round_up(piece_index, 8);
        let mut shift = 7;
        if byte_end > piece_index {
            shift = byte_end - 1 - piece_index;
        }
        1 << shift
    }

    pub fn contains(&self, piece_index: usize) -> bool {
        let byte = self.bitfield[piece_index / 8];
        let mask = self.piece_bit_mask(piece_index);
        (byte & mask) != 0
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&u32::to_be_bytes(self.len))?;
        stream.write_all(&[self.id])?;
        stream.write_all(&self.bitfield)?;
        Ok(())
    }
}
