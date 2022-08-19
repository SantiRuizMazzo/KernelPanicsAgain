use crate::client::download::peer_protocol::ProtocolError;
use std::{
    array::TryFromSliceError,
    io::{Error, Write},
    net::TcpStream,
};

pub const CANCEL_ID: u8 = 8;
const CANCEL_LEN: u32 = 13;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Cancel {
    len: u32,
    id: u8,
    index: u32,
    begin: u32,
    length: u32,
}

impl Cancel {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            len: CANCEL_LEN,
            id: CANCEL_ID,
            index,
            begin,
            length,
        }
    }

    pub fn from(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        let err = |_e: TryFromSliceError| {
            ProtocolError::Peer("Conversion error for Cancel message".to_string())
        };
        let index = u32::from_be_bytes(bytes[1..5].try_into().map_err(err)?);
        let begin = u32::from_be_bytes(bytes[5..9].try_into().map_err(err)?);
        let length = u32::from_be_bytes(bytes[9..13].try_into().map_err(err)?);
        Ok(Self::new(index, begin, length))
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        let size = self.length as i32 - self.begin as i32;
        if size <= 16384 {
            let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
            stream.write_all(&self.len.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.id.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.index.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.begin.to_be_bytes()).map_err(err)?;
            stream.write_all(&self.length.to_be_bytes()).map_err(err)?;
        }
        Ok(())
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn begin(&self) -> u32 {
        self.begin
    }
}
