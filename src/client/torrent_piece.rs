#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct TorrentPiece {
    index: usize,
    length: usize,
    hash: [u8; 20],
    torrent_index: usize,
}

impl TorrentPiece {
    pub fn new(index: usize, length: usize, hash: [u8; 20]) -> TorrentPiece {
        TorrentPiece {
            index,
            length,
            hash,
            torrent_index: 0,
        }
    }

    pub fn set_torrent_index(&mut self, index: usize) {
        self.torrent_index = index;
    }
    pub fn get_torrent_index(&self) -> usize {
        self.torrent_index
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
