use crate::client::download::peer_protocol::ProtocolError;
use std::{
    io::{Error, Write},
    net::TcpStream,
};

pub const HAVE_ID: u8 = 4;
const HAVE_LEN: u32 = 5;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Have {
    len: u32,
    id: u8,
    index: u32,
}

impl Have {
    pub fn new(index: u32) -> Self {
        Self {
            len: HAVE_LEN,
            id: HAVE_ID,
            index,
        }
    }

    pub fn from(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        let piece_index =
            u32::from_be_bytes(bytes[1..].try_into().map_err(|_| {
                ProtocolError::Peer("Conversion error for Have message".to_string())
            })?);
        Ok(Self::new(piece_index))
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
        stream.write_all(&self.len.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.id.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.index.to_be_bytes()).map_err(err)
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}
