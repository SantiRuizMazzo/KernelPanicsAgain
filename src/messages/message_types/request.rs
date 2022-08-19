use super::{block::Block, cancel::Cancel};
use crate::client::download::peer_protocol::{ProtocolError, BLOCK_SIZE};
use std::{
    array::TryFromSliceError,
    io::{Error, Write},
    net::TcpStream,
};

pub const REQUEST_ID: u8 = 6;
const REQUEST_LEN: u32 = 13;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Request {
    len: u32,
    id: u8,
    index: u32,
    begin: u32,
    length: u32,
    sent: bool,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            len: REQUEST_LEN,
            id: REQUEST_ID,
            index: 0,
            begin: 0,
            length: BLOCK_SIZE,
            sent: false,
        }
    }
}

impl Request {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            len: REQUEST_LEN,
            id: REQUEST_ID,
            index,
            begin,
            length,
            sent: false,
        }
    }

    pub fn from(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        let err = |_e: TryFromSliceError| {
            ProtocolError::Peer("Conversion error for Request message".to_string())
        };
        let index = u32::from_be_bytes(bytes[1..5].try_into().map_err(err)?);
        let begin = u32::from_be_bytes(bytes[5..9].try_into().map_err(err)?);
        let length = u32::from_be_bytes(bytes[9..13].try_into().map_err(err)?);
        Ok(Self::new(index, begin, length))
    }

    pub fn send(&mut self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        if self.sent {
            return Ok(());
        }

        let size = self.length as i32 - self.begin as i32;
        if size <= 16384 {
            let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
            stream.write_all(&self.len.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.id.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.index.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.begin.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.length.to_be_bytes()).map_err(err)?;
        }
        self.sent = true;
        Ok(())
    }

    pub fn matches(&self, block: &Block) -> bool {
        block.index() == self.index
            && block.begin() == self.begin
            && block.len() == self.length as usize
    }

    pub fn load_block_from(&self, path: String) -> Result<Block, String> {
        let mut block = Block::new(self.index, self.begin, vec![0; self.length as usize]);
        block.load_from(path)?;
        Ok(block)
    }

    pub fn reset(&mut self) {
        self.sent = false
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn cancel(&self) -> Cancel {
        Cancel::new(self.index, self.begin, self.length)
    }
}
