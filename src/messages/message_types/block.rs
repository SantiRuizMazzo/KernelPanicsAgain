use std::{
    array::TryFromSliceError,
    fs::File,
    io::{Error, Read, Seek, SeekFrom, Write},
    net::TcpStream,
};

use crate::client::download::peer_protocol::ProtocolError;

pub const BLOCK_ID: u8 = 7;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Block {
    len: u32,
    id: u8,
    index: u32,
    begin: u32,
    block: Vec<u8>,
}

impl Block {
    pub fn new(index: u32, begin: u32, block: Vec<u8>) -> Self {
        Self {
            len: block.len() as u32 + 9,
            id: BLOCK_ID,
            index,
            begin,
            block,
        }
    }

    pub fn from(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        let err = |_e: TryFromSliceError| {
            ProtocolError::Peer("Conversion error for Block message".to_string())
        };
        let index = u32::from_be_bytes(bytes[1..5].try_into().map_err(err)?);
        let begin = u32::from_be_bytes(bytes[5..9].try_into().map_err(err)?);
        let block = Vec::from(&bytes[9..]);
        Ok(Self::new(index, begin, block))
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
        stream.write_all(&self.len.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.id.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.index.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.begin.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.block).map_err(err)
    }

    pub fn load_from(&mut self, path: String) -> Result<(), String> {
        let err = |e: Error| e.to_string();
        let block_path = format!("{path}/.tmp/{}", self.index);
        let mut file = File::open(block_path).map_err(err)?;
        file.seek(SeekFrom::Start(self.begin as u64)).map_err(err)?;
        file.read_exact(&mut self.block).map_err(err)
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn begin(&self) -> u32 {
        self.begin
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.block.clone()
    }

    pub fn len(&self) -> usize {
        self.block.len()
    }

    pub fn next_begin(&self) -> usize {
        self.begin as usize + self.len()
    }
}
