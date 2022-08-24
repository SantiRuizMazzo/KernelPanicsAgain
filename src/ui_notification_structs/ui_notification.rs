use super::torrent_state::TorrentState;

impl UiNotification {
    pub fn new() -> UiNotification {
        UiNotification {
            torrents: Vec::new(),
        }
    }

    pub fn add_torrent_state(&mut self, torrent: TorrentState) {
        self.torrents.push(torrent);
    }

    pub fn get_state_by_torrent_name(&self, name: &str) -> Option<TorrentState> {
        let mut result: Option<TorrentState> = None;
        for torrent in &self.torrents {
            if torrent.get_metadata_name().eq(name) {
                result = Some(torrent.clone());
            }
        }
        result
    }

    pub fn get_torrent_states(&self) -> Vec<TorrentState> {
        self.torrents.clone()
    }
}
impl Default for UiNotification {
    fn default() -> Self {
        Self::new()
    }
}
pub struct UiNotification {
    torrents: Vec<TorrentState>,
}
