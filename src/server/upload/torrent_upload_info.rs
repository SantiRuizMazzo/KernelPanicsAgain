use crate::messages::message_type::bitfield::Bitfield;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TorrentUploadInfo {
    info_hash: [u8; 20],
    download_path: String,
    bitfield: Option<Bitfield>,
    pieces_qty: u32,
}

impl TorrentUploadInfo {
    pub fn new(info_hash: [u8; 20], download_path: String, pieces_qty: u32) -> TorrentUploadInfo {
        TorrentUploadInfo {
            info_hash,
            download_path,
            bitfield: None,
            pieces_qty,
        }
    }

    pub fn get_hash(&self) -> [u8; 20] {
        self.info_hash
    }

    pub fn get_path(&self) -> String {
        self.download_path.clone()
    }

    pub fn add_piece_to_bitfield(&mut self, piece_index: usize) -> Result<(), String> {
        let mut bitfield = match self.bitfield.clone() {
            Some(msg) => msg,
            None => {
                let mut bitfield_vec = Vec::<u8>::new();
                let mut byte_amount = self.pieces_qty / 8;

                if self.pieces_qty % 8 > 0 {
                    byte_amount += 1;
                }

                for _i in 0..byte_amount {
                    bitfield_vec.push(0);
                }

                Bitfield::new(bitfield_vec.clone())
            }
        };

        bitfield.set_have_piece(piece_index as usize);
        self.set_bitfield(bitfield);
        Ok(())
    }

    pub fn set_bitfield(&mut self, bitfield: Bitfield) {
        self.bitfield = Some(bitfield);
    }

    pub fn get_bitfield(&self) -> Option<Bitfield> {
        self.bitfield.clone()
    }
}
