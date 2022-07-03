#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct DownloadInfo {
    client_id: [u8; 20],
    info_hash: [u8; 20],
}

impl DownloadInfo {
    pub fn new(client_id: [u8; 20], info_hash: [u8; 20]) -> DownloadInfo {
        DownloadInfo {
            client_id,
            info_hash,
        }
    }

    pub fn get_id(&self) -> [u8; 20] {
        self.client_id
    }

    pub fn get_hash(&self) -> [u8; 20] {
        self.info_hash
    }
}
