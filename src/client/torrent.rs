use super::{single_file::SingleFile, tracker_info::TrackerInfo};

/// Stores the information that a .torrent file contains.

pub struct Torrent {
    pub announce: String,
    pub piece_length: i32,
    pub pieces: Vec<u8>,
    pub files: Vec<SingleFile>,
    pub tracker_info: TrackerInfo,
    pub info_hash: [u8; 20],
}
