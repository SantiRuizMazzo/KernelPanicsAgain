#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DownloadInfo {
    client_id: [u8; 20],
    info_hash: [u8; 20],
    path: String,
    pieces_qty: usize,
}

impl DownloadInfo {
    pub fn new(
        client_id: [u8; 20],
        info_hash: [u8; 20],
        path: String,
        pieces_qty: usize,
    ) -> DownloadInfo {
        DownloadInfo {
            client_id,
            info_hash,
            path,
            pieces_qty,
        }
    }

    pub fn get_id(&self) -> [u8; 20] {
        self.client_id
    }

    pub fn get_hash(&self) -> [u8; 20] {
        self.info_hash
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }
    pub fn get_pieces_qty(&self) -> usize {
        self.pieces_qty
    }
}
