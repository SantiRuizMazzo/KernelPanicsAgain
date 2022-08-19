use std::cmp;

use crate::{
    messages::message_types::{block::Block, request::Request},
    utils,
};

use super::download::peer_protocol::BLOCK_SIZE;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Piece {
    index: usize,
    bytes: Vec<u8>,
    hash: [u8; 20],
    next_block_begin: usize,
}

impl Piece {
    pub fn new(index: usize, length: usize, hash: [u8; 20]) -> Self {
        Self {
            index,
            bytes: Vec::with_capacity(length),
            hash,
            next_block_begin: 0,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn bytes_left(&self) -> usize {
        self.bytes.capacity() - self.bytes.len()
    }

    pub fn is_full(&self) -> bool {
        self.bytes_left() == 0
    }

    pub fn hash(&self) -> [u8; 20] {
        self.hash
    }

    pub fn hashes_match(&self) -> bool {
        let mut new_hash = [0; 20];
        if let Ok(hash) = utils::sha1(&self.bytes) {
            new_hash = hash
        };
        self.hash == new_hash
    }

    pub fn append(&mut self, block: &Block) {
        self.bytes.append(&mut block.bytes());
        self.next_block_begin = block.next_begin()
    }

    pub fn request_next_block(&self) -> Request {
        let length = cmp::min(BLOCK_SIZE, self.bytes_left() as u32);
        Request::new(self.index as u32, self.next_block_begin as u32, length)
    }
}
