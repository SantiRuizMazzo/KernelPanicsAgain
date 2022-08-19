use std::{
    io::{Error, Write},
    net::TcpStream,
};

use crate::client::download::peer_protocol::ProtocolError;

pub const UNCHOKE_ID: u8 = 1;
const UNCHOKE_LEN: u32 = 1;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Unchoke {
    len: u32,
    id: u8,
}

impl Unchoke {
    pub fn new() -> Self {
        Self {
            len: UNCHOKE_LEN,
            id: UNCHOKE_ID,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<(), ProtocolError> {
        let err = |e: Error| ProtocolError::Peer(format!("Failed sending {self:?} ({e})"));
        stream.write_all(&self.len.to_be_bytes()).map_err(err)?;
        stream.write_all(&self.id.to_be_bytes()).map_err(err)
    }
}
