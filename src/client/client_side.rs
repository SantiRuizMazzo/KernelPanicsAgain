use crate::config::Config;
use crate::{
    client::torrent::Torrent, logger::torrent_logger::LogMessage, utils::ServerNotification,
};
use rand::Rng;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::{fs, ops::Deref, path::Path};

use super::download::download_pool::DownloadPool;

pub type TorrentSender = Sender<Torrent>;
pub type TorrentReceiver = Arc<Mutex<Receiver<Torrent>>>;
pub type DownloadedTorrents = Arc<Mutex<Vec<Torrent>>>;

#[derive(Clone)]
pub struct ClientSide {
    pub peer_id: [u8; 20],
    pub config: Config,
    notification_tx: Sender<ServerNotification>,
    torrent_queue: (TorrentSender, TorrentReceiver),
    downloaded_torrents: DownloadedTorrents,
}

impl ClientSide {
    pub fn new(
        config: Config,
        notification_tx: Sender<ServerNotification>,
    ) -> Result<ClientSide, String> {
        let (torrent_tx, torrent_rx) = mpsc::channel::<Torrent>();
        let torrent_queue = (torrent_tx, Arc::new(Mutex::new(torrent_rx)));
        let downloaded_torrents = Arc::new(Mutex::new(Vec::<Torrent>::new()));

        Ok(ClientSide {
            peer_id: ClientSide::generate_peer_id()?,
            config,
            notification_tx,
            torrent_queue,
            downloaded_torrents,
        })
    }

    fn generate_peer_id() -> Result<[u8; 20], String> {
        let mut peer_id = b"-PK0001-".to_vec();
        let mut generator = rand::thread_rng();
        for _i in 0..12 {
            let aux: u8 = generator.gen_range(0..10);
            peer_id.push(aux)
        }
        peer_id
            .try_into()
            .map_err(|_| "conversion error".to_string())
    }

    pub fn get_id(&self) -> [u8; 20] {
        self.peer_id
    }

    pub fn load_torrents<A>(&mut self, args: A) -> Result<(), String>
    where
        A: Iterator<Item = String>,
    {
        for arg in args {
            let path = Path::new(&arg);
            if path.is_dir() {
                self.load_from_dir(path)?
            } else if path.is_file() {
                self.load_from_file(path)?
            }
        }
        Ok(())
    }

    fn load_from_dir(&mut self, dir: &Path) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
            let path = entry.map_err(|err| err.to_string())?.path();
            self.load_from_file(path)?;
        }
        Ok(())
    }

    fn load_from_file<F>(&mut self, file: F) -> Result<(), String>
    where
        F: Deref<Target = Path> + AsRef<Path>,
    {
        if let Some(extension) = file.extension() {
            if extension == "torrent" {
                self.torrent_queue
                    .0
                    .send(Torrent::from(file)?)
                    .map_err(|err| err.to_string())?
            }
        }
        Ok(())
    }

    pub fn init(&mut self, logger_tx: Sender<LogMessage>) -> Result<(), String> {
        let pool = DownloadPool::new(
            self.torrent_queue.clone(),
            self.downloaded_torrents.clone(),
            logger_tx,
            self.peer_id,
            &self.config,
            self.notification_tx.clone(),
        );
        pool.ids();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn generate_correctly_sized_peer_id() -> Result<(), String> {
        let s = ClientSide::generate_peer_id()?;
        assert_eq!(20, s.len() * std::mem::size_of::<u8>());
        Ok(())
    }

    #[test]
    fn generate_correctly_sized_peer_id_inside_client_side_struct() -> Result<(), String> {
        let (tx, _rx) = mpsc::channel();
        let client = ClientSide::new(Config::new()?, tx)?;
        assert_eq!(20, client.peer_id.len() * std::mem::size_of::<u8>());
        Ok(())
    }

    #[test]
    fn client_generator() -> Result<(), String> {
        let (tx, _rx) = mpsc::channel();
        let client = ClientSide::new(Config::new()?, tx)?;
        assert_eq!(20, client.peer_id.len() * std::mem::size_of::<u8>());
        Ok(())
    }

    #[test]
    fn load_a_single_torrent_from_a_path_to_file() -> Result<(), String> {
        let (tx, _rx) = mpsc::channel();
        let mut client = ClientSide::new(Config::new()?, tx)?;
        let command_line_args = vec!["tests/debian.torrent".to_string()].into_iter();
        client.load_torrents(command_line_args)?;
        /*assert_eq!(
            vec![Torrent::from("tests/debian.torrent")?],
            client.torrents
        );*/
        Ok(())
    }

    #[test]
    fn load_multiple_torrents_from_multiple_paths_to_files() -> Result<(), String> {
        let (tx, _rx) = mpsc::channel();
        let mut client = ClientSide::new(Config::new()?, tx)?;
        let command_line_args = vec![
            "tests/debian.torrent".to_string(),
            "tests/fedora.torrent".to_string(),
            "tests/linuxmint.torrent".to_string(),
        ]
        .into_iter();
        client.load_torrents(command_line_args)?;
        /*let expected_torrents = vec![
            Torrent::from("tests/debian.torrent")?,
            Torrent::from("tests/fedora.torrent")?,
            Torrent::from("tests/linuxmint.torrent")?,
        ];
        assert_eq!(expected_torrents, client.torrents);*/
        Ok(())
    }

    #[test]
    fn load_multiple_torrents_from_a_path_to_directory() -> Result<(), String> {
        let (tx, _rx) = mpsc::channel();
        let mut client = ClientSide::new(Config::new()?, tx)?;
        let command_line_args = vec!["tests".to_string()].into_iter();
        client.load_torrents(command_line_args)?;
        /*assert!(client
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
        assert_eq!(6, client.torrents.len());*/
        Ok(())
    }

    #[test]
    fn load_torrents_from_path_to_directory_without_torrents_should_fail() -> Result<(), String> {
        let (tx, _rx) = mpsc::channel();
        let mut client = ClientSide::new(Config::new()?, tx)?;
        let args = vec!["src".to_string()].into_iter();
        assert!(client.load_torrents(args).is_err());
        Ok(())
    }
}
