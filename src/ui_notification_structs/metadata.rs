impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            info_hash: [0; 20],
            total_size: 0,
            n_pieces: 1,
            is_single: true,
            downloaded: 0,
            conections: 0,
            name: "".to_string(),
        }
    }

    pub fn set_info_hash(&mut self, _info_hash: [u8; 20]) {
        self.info_hash = _info_hash;
    }

    pub fn get_info_hash(&self) -> [u8; 20] {
        self.info_hash
    }

    pub fn set_total_size(&mut self, total_size: u32) {
        self.total_size = total_size;
    }

    pub fn get_total_size(&self) -> u32 {
        self.total_size
    }

    pub fn set_n_pieces(&mut self, n_pieces: u32) {
        self.n_pieces = n_pieces;
    }

    pub fn get_n_pieces(&self) -> u32 {
        self.n_pieces
    }

    pub fn set_is_single(&mut self, _is_single: bool) {
        self.is_single = _is_single;
    }

    pub fn get_is_single(&self) -> bool {
        self.is_single
    }

    pub fn set_downloaded(&mut self, _downloaded: u32) {
        self.downloaded = _downloaded;
    }

    pub fn get_downloaded(&self) -> u32 {
        self.downloaded
    }

    pub fn set_connections(&mut self, _conections: u32) {
        self.conections = _conections;
    }

    pub fn get_connections(&self) -> u32 {
        self.conections
    }

    pub fn set_name(&mut self, _name: String) {
        self.name = _name;
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone)]
pub struct Metadata {
    info_hash: [u8; 20],
    total_size: u32,
    n_pieces: u32,
    is_single: bool,
    downloaded: u32,
    conections: u32,
    name: String,
}
