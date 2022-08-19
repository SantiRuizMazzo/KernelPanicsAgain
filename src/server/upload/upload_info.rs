use crate::messages::message_types::bitfield::Bitfield;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UploadInfo {
    info_hash: [u8; 20],
    download_path: String,
    bitfield: Option<Bitfield>,
    total_pieces: usize,
}

impl UploadInfo {
    pub fn new(info_hash: [u8; 20], download_path: String, total_pieces: usize) -> Self {
        Self {
            info_hash,
            download_path,
            bitfield: None,
            total_pieces,
        }
    }

    pub fn info_hash(&self) -> [u8; 20] {
        self.info_hash
    }

    pub fn download_path(&self) -> String {
        self.download_path.clone()
    }

    pub fn add_piece_to_bitfield(&mut self, piece_index: usize) -> Result<(), String> {
        let mut bitfield = match self.bitfield.clone() {
            Some(msg) => msg,
            None => {
                let mut bitfield_vec = Vec::<u8>::new();
                let mut byte_amount = self.total_pieces / 8;

                if self.total_pieces % 8 > 0 {
                    byte_amount += 1;
                }

                for _i in 0..byte_amount {
                    bitfield_vec.push(0);
                }

                Bitfield::new(bitfield_vec.clone())
            }
        };

        bitfield.add_piece(piece_index as usize);
        self.bitfield = Some(bitfield);
        Ok(())
    }

    pub fn bitfield(&self) -> Option<Bitfield> {
        self.bitfield.clone()
    }
}
