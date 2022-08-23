use chrono::Local;
use serde_json::{Map, Value};

use crate::server::tracker_info::torrent_tracker_info::TorrentTrackerData;
use std::{
    cmp,
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    sync::{Arc, Mutex},
};

use super::peer_tracker_info::PeerTrackerInfo;

type TrackedTorrentsHash = Arc<Mutex<HashMap<String, TorrentTrackerData>>>;

#[derive(Debug, Clone)]
pub struct TorrentRegistry {
    tracked_torrents_mutex: TrackedTorrentsHash,
}

impl TorrentRegistry {
    pub fn new() -> TorrentRegistry {
        let tracked_torrents_mutex =
            Arc::new(Mutex::new(HashMap::<String, TorrentTrackerData>::new()));
        TorrentRegistry {
            tracked_torrents_mutex,
        }
    }

    pub fn insert(&self, info_hash: String, peer: PeerTrackerInfo) {
        if let Ok(mut tracked_torrents) = self.tracked_torrents_mutex.lock() {
            let torrent = tracked_torrents.get_mut(&info_hash);
            
            let torrent_to_insert = match torrent {
                Some(tracked_torrent) => {
                    tracked_torrent.insert_peer(peer);
                    tracked_torrent.clone()
                }
                None => {
                    let peers_hash = HashMap::<String, PeerTrackerInfo>::new();
                    let mut new_torrent_data =
                        TorrentTrackerData::new(info_hash.clone(), peers_hash);
                    new_torrent_data.insert_peer(peer);
                    new_torrent_data
                }
            };
            tracked_torrents.insert(info_hash, torrent_to_insert);
        };
    }

    fn get_info_json(&self) -> Result<serde_json::Value, String> {
        let mut torrents_map = Map::new();
        if let Ok(tracked_torrents) = self.tracked_torrents_mutex.lock() {
            for (key, tracked_torrent_data) in tracked_torrents.iter() {
                let mut torrent_info_map = Map::new();
                let mut value = serde_json::to_value(tracked_torrent_data.get_connected_peers())
                    .map_err(|err| err.to_string())?;
                torrent_info_map.insert("conectados".to_string(), value);
                value = serde_json::to_value(tracked_torrent_data.get_completed_peers())
                    .map_err(|err| err.to_string())?;
                torrent_info_map.insert("completos".to_string(), value);
                value = serde_json::to_value(torrent_info_map).map_err(|err| err.to_string())?;
                torrents_map.insert(key.clone(), value);
            }
        }
        serde_json::to_value(torrents_map).map_err(|err| err.to_string())
    }

    pub fn get_bencoded_contents(
        &self,
        info_hash: String,
        numwant: usize,
    ) -> Result<Vec<u8>, String> {
        let tracked_torrents = self
            .tracked_torrents_mutex
            .lock()
            .map_err(|err| format!("Mutex lock error: {}", err))?;

        match tracked_torrents.get(&info_hash) {
            Some(torrent) => {
                let complete = torrent.get_completed_peers();
                let incomplete = torrent.get_connected_peers() - complete;
                let peers = torrent.get_peers_bytes();
                let min = cmp::min(numwant * 6, peers.len());
                let mut peers = peers[..min].to_vec();
                let mut content = format!(
                    "d8:intervali900e8:completei{complete}e10:incompletei{incomplete}e5:peers{}:",
                    peers.len()
                )
                .as_bytes()
                .to_vec();
                content.append(&mut peers);
                content.append(&mut b"e".to_vec());
                Ok(content)
            }
            None => Ok("d14:failure reason44:Torrent Not Offered. Added you as first peere".to_string()
            .as_bytes()
            .to_vec()),
        }
    }

    pub fn save_to_json(&self) -> Result<(), String> {
        let now = Local::now().timestamp();
        let three_days_as_seconds: i64 = 259200;
        let file = match File::open("./tracker_server/server/data.json") {
                Ok(file ) => file,
                Err(_) => {
                    let new_file = File::create("./tracker_server/server/data.json");
                    match new_file {
                        Ok(file) => file,
                        Err(_) => {return Err("Error while creating data.json".to_string());}
                    }

            }
        };
        let reader = BufReader::new(file);
        let json_data: Value = match serde_json::from_reader::<BufReader<File>, Value>(reader){
            Ok(json) => json,
            Err(_) => Value::Object(Map::new()),
        };
        if let Some(map) = json_data.as_object() {
            let mut new_map = map.clone();
            for (json_timestamp, _) in map {
                let json_timestamp_aux = json_timestamp
                    .parse::<i64>()
                    .map_err(|err| err.to_string())?;
                if three_days_as_seconds + json_timestamp_aux < now {
                    let _ = new_map.remove(json_timestamp);
                }
            }
            let now = Local::now().timestamp().to_string();
            new_map.insert(now, self.get_info_json().map_err(|err| err)?);
            let updated_json_data = serde_json::to_value(new_map).map_err(|err| err.to_string())?;
            let string =
                serde_json::to_string_pretty(&updated_json_data).map_err(|err| err.to_string())?;
            let _ = fs::write("./tracker_server/server/data.json", string);
        }
        Ok(())
    }
}
