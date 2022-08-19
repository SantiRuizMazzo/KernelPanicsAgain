use std::{
    io::{Error, Write},
    net::TcpStream,
};

use crate::client::download::peer_protocol::ProtocolError;

pub const INTERESTED_ID: u8 = 2;
const INTERESTED_LEN: u32 = 1;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Interested {
    len: u32,
    id: u8,
}

impl Interested {
    pub fn new() -> Self {
        Self {
            len: INTERESTED_LEN,
            id: INTERESTED_ID,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
        stream.write_all(&self.len.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.id.to_be_bytes()).map_err(err)
    }
}
