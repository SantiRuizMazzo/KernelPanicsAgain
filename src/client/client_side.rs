use crate::{
    client::torrent::Torrent, config::Config, logging::log_handle::LogHandle,
    server::server_side::Notification,
};
use rand::Rng;
use std::{
    env,
    io::Error,
    sync::{
        mpsc::{self, Receiver, Sender},
        {Arc, Mutex},
    },
    {fs, ops::Deref, path::Path},
};

use super::download::download_pool::DownloadPool;

const TORRENT_EXTENSION: &str = "torrent";

pub type TorrentSender = Sender<Torrent>;
pub type TorrentReceiver = Arc<Mutex<Receiver<Torrent>>>;
pub type DownloadedTorrents = Arc<Mutex<Vec<Torrent>>>;

#[derive(Clone)]
pub struct ClientSide {
    id: [u8; 20],
    config: Config,
    torrent_tx: TorrentSender,
    torrent_rx: TorrentReceiver,
    downloaded_torrents: DownloadedTorrents,
    log_handle: LogHandle,
}

impl ClientSide {
    pub fn new(config: &Config, log_handle: LogHandle) -> Self {
        let (torrent_tx, torrent_rx) = mpsc::channel();

        Self {
            id: Self::generate_id(),
            config: config.clone(),
            torrent_tx,
            torrent_rx: Arc::new(Mutex::new(torrent_rx)),
            downloaded_torrents: Arc::new(Mutex::new(Vec::new())),
            log_handle,
        }
    }

    fn generate_id() -> [u8; 20] {
        let mut id = *b"-PK0001-000000000000";
        let mut generator = rand::thread_rng();

        for index in id.iter_mut().skip(8) {
            *index = generator.gen_range(0..10)
        }
        id
    }

    pub fn get_id(&self) -> [u8; 20] {
        self.id
    }

    pub fn load_torrents<A>(&mut self, paths: A) -> Result<(), String>
    where
        A: IntoIterator<Item = String>,
    {
        for path in paths {
            let path = Path::new(&path);
            if path.is_dir() {
                self.load_from_dir(path)?
            } else if path.is_file() {
                self.load_from_file(path)?
            }
        }
        Ok(())
    }

    fn load_from_dir(&mut self, dir: &Path) -> Result<(), String> {
        let err = |e: Error| e.to_string();
        for entry in fs::read_dir(dir).map_err(err)? {
            let path = entry.map_err(err)?.path();
            self.load_from_file(path)?;
        }
        Ok(())
    }

    fn load_from_file<F>(&mut self, file: F) -> Result<(), String>
    where
        F: Deref<Target = Path> + AsRef<Path>,
    {
        if let Some(extension) = file.extension() {
            if extension == TORRENT_EXTENSION {
                let mut torrent = Torrent::from(file)?;
                torrent.save_in(self.config.download_path());
                self.torrent_tx.send(torrent).map_err(|e| e.to_string())?
            }
        }
        Ok(())
    }

    pub fn init(&mut self, notif_tx: Sender<Notification>) -> Result<(), String> {
        self.load_torrents(env::args())?;

        DownloadPool::new(
            self.id,
            &self.config,
            &self.torrent_tx,
            &self.torrent_rx,
            &self.downloaded_torrents,
            notif_tx,
            &self.log_handle,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::logger::Logger;

    #[test]
    fn generate_correctly_sized_id() {
        let id = ClientSide::generate_id();
        assert_eq!(20, id.len());
    }

    #[test]
    fn generate_correctly_sized_id_inside_client_side_struct() -> Result<(), String> {
        let config = Config::new()?;
        let logger = Logger::new(config.log_path())?;
        let client = ClientSide::new(&config, logger.handle());
        assert_eq!(20, client.id.len());
        Ok(())
    }

    /*#[test]
    fn load_a_single_torrent_from_a_path_to_file() -> Result<(), String> {
        let config = Config::new()?;
        let logger = Logger::new(config.get_log_path())?;
        let mut client = ClientSide::new(&config, logger.handle());
        let paths = vec!["tests/debian.torrent".to_string()];
        client.load_torrents(paths)?;
        assert_eq!(
            vec![Torrent::from("tests/debian.torrent")?],
            client.torrents
        );
        Ok(())
    }

    #[test]
    fn load_multiple_torrents_from_multiple_paths_to_files() -> Result<(), String> {
        let config = Config::new()?;
        let logger = Logger::new(config.get_log_path())?;
        let mut client = ClientSide::new(&config, logger.handle());
        let paths = vec![
            "tests/debian.torrent".to_string(),
            "tests/fedora.torrent".to_string(),
            "tests/linuxmint.torrent".to_string(),
        ];
        client.load_torrents(paths)?;
        let expected_torrents = vec![
            Torrent::from("tests/debian.torrent")?,
            Torrent::from("tests/fedora.torrent")?,
            Torrent::from("tests/linuxmint.torrent")?,
        ];
        assert_eq!(expected_torrents, client.torrents);
        Ok(())
    }

    #[test]
    fn load_multiple_torrents_from_a_path_to_directory() -> Result<(), String> {
        let config = Config::new()?;
        let logger = Logger::new(config.get_log_path())?;
        let mut client = ClientSide::new(&config, logger.handle());
        let paths = vec!["tests".to_string()];
        client.load_torrents(paths)?;
        assert!(client
            .torrents
            .contains(&Torrent::from("tests/bla.torrent")?));
        assert!(client
            .torrents
            .contains(&Torrent::from("tests/fedora.torrent")?));
        assert!(client
            .torrents
            .contains(&Torrent::from("tests/debian.torrent")?));
        assert!(client
            .torrents
            .contains(&Torrent::from("tests/linuxmint.torrent")?));
        assert!(client
            .torrents
            .contains(&Torrent::from("tests/sample.torrent")?));
        assert!(client
            .torrents
            .contains(&Torrent::from("tests/lubuntu.torrent")?));
        assert_eq!(6, client.torrents.len());
        Ok(())
    }*/

    #[test]
    fn load_torrents_from_path_to_directory_without_torrents_should_fail() -> Result<(), String> {
        let config = Config::new()?;
        let logger = Logger::new(config.log_path())?;
        let mut client = ClientSide::new(&config, logger.handle());
        let paths = vec!["src".to_string()];
        assert!(client.load_torrents(paths).is_err());
        Ok(())
    }
}
