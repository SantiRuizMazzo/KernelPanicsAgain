/// Stores information about each file that is meant to be downloaded from a multiple-file .torrent file.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SingleFile {
    pub length: i64,
    pub path: String,
}

impl SingleFile {
    pub fn new(length: i64, path: String) -> SingleFile {
        SingleFile { length, path }
    }
}
