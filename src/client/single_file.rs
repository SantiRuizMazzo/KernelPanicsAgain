/// Stores information about each file that is meant to be downloaded from a multiple-file .torrent file.

pub struct SingleFile {
    pub length: i64,
    pub path: String,
}
