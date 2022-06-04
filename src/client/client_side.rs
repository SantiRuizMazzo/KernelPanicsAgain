use super::{peer::Peer, single_file::SingleFile, torrent::Torrent};
use crate::urlencoding::encode;
use rand::Rng;
use std::{fs, ops::Deref, path::Path};

pub struct ClientSide {
    pub peer_id: String,
    torrents: Vec<Torrent>,
}

impl ClientSide {
    fn generate_peer_id() -> String {
        let mut peer_id = String::from("-PK0001-");
        let mut generator = rand::thread_rng();
        for _i in 0..12 {
            let aux: i8 = generator.gen_range(0..10);
            peer_id += &aux.to_string();
        }
        peer_id
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
        if self.torrents.is_empty() {
            return Err("could not load any .torrent files".to_string());
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
                self.torrents.push(Torrent::from(file)?)
            }
        }
        Ok(())
    }

    pub fn new() -> ClientSide {
        let torrent = Torrent::from("tests/ultramarine.torrent").unwrap();
        let tracker_info = torrent
            .get_tracker_info(*b"12345678901234567890", 6881)
            .unwrap();
        tracker_info.print();
        let _ = SingleFile::new(0, "xd.txt".to_string());
        let peer = Peer::new(None, "chau".to_string(), 0);
        peer.print();
        let _ = encode("上海+中國");

        ClientSide {
            peer_id: ClientSide::generate_peer_id(),
            torrents: Vec::new(),
        }
    }
}

/*-<2 letras><4 numeros de version>-<12 numeros random>*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correctly_sized_peer_id() {
        let s = ClientSide::generate_peer_id();
        assert_eq!(20, s.len() * std::mem::size_of::<u8>());
    }

    #[test]
    fn generate_correctly_sized_peer_id_inside_clientside_struct() {
        let client = ClientSide::new();
        assert_eq!(20, client.peer_id.len() * std::mem::size_of::<u8>());
    }

    #[test]
    fn load_a_single_torrent_from_a_path_to_file() -> Result<(), String> {
        let mut client = ClientSide::new();
        let command_line_args = vec!["tests/debian.torrent".to_string()].into_iter();
        client.load_torrents(command_line_args)?;
        assert_eq!(
            vec![Torrent::from("tests/debian.torrent")?],
            client.torrents
        );
        Ok(())
    }

    #[test]
    fn load_multiple_torrents_from_multiple_paths_to_files() -> Result<(), String> {
        let mut client = ClientSide::new();
        let command_line_args = vec![
            "tests/debian.torrent".to_string(),
            "tests/fedora.torrent".to_string(),
            "tests/linuxmint.torrent".to_string(),
        ]
        .into_iter();
        client.load_torrents(command_line_args)?;
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
        let mut client = ClientSide::new();
        let command_line_args = vec!["tests".to_string()].into_iter();
        client.load_torrents(command_line_args)?;
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
            .contains(&Torrent::from("tests/ubuntu.torrent")?));
        assert!(client
            .torrents
            .contains(&Torrent::from("tests/ultramarine.torrent")?));
        assert_eq!(7, client.torrents.len());
        Ok(())
    }

    #[test]
    fn load_torrents_from_path_to_directory_without_torrents_should_fail() -> Result<(), String> {
        let mut client = ClientSide::new();
        let args = vec!["src".to_string()].into_iter();
        assert!(client.load_torrents(args).is_err());
        Ok(())
    }
}
