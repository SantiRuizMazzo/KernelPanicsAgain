use std::time::Instant;

impl PeerState {
    pub fn new(last_download_instant: Option<Instant>) -> PeerState {
        PeerState {
            id: "".to_string(),
            ip: "".to_string(),
            prot: 0,
            p_is_chocked: false,
            p_is_interested: true,
            c_is_chocked: false,
            c_is_interested: true,
            last_download_instant,
        }
    }

    pub fn set_id(&mut self, _id: String) {
        self.id = _id;
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn set_ip(&mut self, _ip: String) {
        self.ip = _ip;
    }

    pub fn get_ip(&self) -> String {
        self.ip.clone()
    }

    pub fn set_port(&mut self, _prot: u16) {
        self.prot = _prot;
    }

    pub fn get_port(&self) -> u16 {
        self.prot
    }

    pub fn set_p_is_chocked(&mut self, _p_is_chocked: bool) {
        self.p_is_chocked = _p_is_chocked;
    }

    pub fn get_p_is_chocked(&self) -> bool {
        self.p_is_chocked
    }

    pub fn set_p_is_interested(&mut self, _p_is_interested: bool) {
        self.p_is_interested = _p_is_interested;
    }

    pub fn get_p_is_interested(&self) -> bool {
        self.p_is_interested
    }

    pub fn set_c_is_chocked(&mut self, _c_is_chocked: bool) {
        self.c_is_chocked = _c_is_chocked;
    }

    pub fn get_c_is_chocked(&self) -> bool {
        self.c_is_chocked
    }

    pub fn set_c_is_interested(&mut self, _c_is_interested: bool) {
        self.c_is_interested = _c_is_interested;
    }

    pub fn get_c_is_interested(&self) -> bool {
        self.c_is_interested
    }

    pub fn set_last_download_instant(&mut self, _last_download_instant: Option<Instant>) {
        self.last_download_instant = _last_download_instant;
    }

    pub fn get_last_download_instant(&self) -> Option<Instant> {
        self.last_download_instant
    }

    pub fn get_download_v(&self, piece_size: u32) -> f64 {
        let mut download_v: f64 = 0.0;
        if let Some(downloaded_instant) = self.last_download_instant {
            let now = Instant::now();
            let time_lapse = now.duration_since(downloaded_instant);
            download_v = (piece_size as f64) / time_lapse.as_secs_f64();
        }
        download_v
    }
}
impl Default for PeerState {
    fn default() -> Self {
        Self::new(Some(Instant::now()))
    }
}
#[derive(Clone)]
pub struct PeerState {
    id: String,
    ip: String,
    prot: u16,
    p_is_chocked: bool,
    p_is_interested: bool,
    c_is_chocked: bool,
    c_is_interested: bool,
    last_download_instant: Option<Instant>,
}
