#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct TorrentPiece {
    index: usize,
    length: usize,
    hash: [u8; 20],
}

impl TorrentPiece {
    pub fn new(index: usize, length: usize, hash: [u8; 20]) -> TorrentPiece {
        TorrentPiece {
            index,
            length,
            hash,
        }
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_hash(&self) -> [u8; 20] {
        self.hash
    }
}
