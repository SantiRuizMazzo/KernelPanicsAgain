use crate::client::download::peer_protocol::ProtocolError;

use super::message_types::{
    bitfield::{Bitfield, BITFIELD_ID},
    block::{Block, BLOCK_ID},
    cancel::{Cancel, CANCEL_ID},
    have::{Have, HAVE_ID},
    interested::INTERESTED_ID,
    request::{Request, REQUEST_ID},
    unchoke::UNCHOKE_ID,
};

const CHOKE_ID: u8 = 0;
const NOT_INTERESTED_ID: u8 = 3;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(Have),
    Bitfield(Bitfield),
    Request(Request),
    Block(Block),
    Cancel(Cancel),
}

impl PeerMessage {
    pub fn from(bytes: Vec<u8>) -> Result<Self, ProtocolError> {
        if bytes.is_empty() {
            return Ok(Self::KeepAlive);
        }
        match bytes[0] {
            CHOKE_ID => Ok(Self::Choke),
            UNCHOKE_ID => Ok(Self::Unchoke),
            INTERESTED_ID => Ok(Self::Interested),
            NOT_INTERESTED_ID => Ok(Self::NotInterested),
            HAVE_ID => Ok(Self::Have(Have::from(bytes)?)),
            BITFIELD_ID => Ok(Self::Bitfield(Bitfield::from(bytes)?)),
            REQUEST_ID => Ok(Self::Request(Request::from(bytes)?)),
            BLOCK_ID => Ok(Self::Block(Block::from(bytes)?)),
            CANCEL_ID => Ok(Self::Cancel(Cancel::from(bytes)?)),
            _ => Err(ProtocolError::Peer(
                "Couldn't parse peer message".to_string(),
            )),
        }
    }
}
