use std::sync::mpsc::Sender;

use crate::{server::upload::torrent_upload_info::TorrentUploadInfo, utils::ServerNotification};

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

    pub fn notify_present(
        &self,
        notification_sender: Sender<ServerNotification>,
        upload_info: TorrentUploadInfo,
    ) -> Result<(), String> {
        notification_sender
            .send(ServerNotification::HavePiece(*self, upload_info))
            .map_err(|err| err.to_string())
    }
}
