use super::{single_file::SingleFile, torrent::Torrent, tracker_info::TrackerInfo};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;

const INT_START: u8 = 0x69;
const INT_END: u8 = 0x65;
const LIST_START: u8 = 0x6C;
const LIST_END: u8 = 0x65;
const DICT_START: u8 = 0x64;
const DICT_END: u8 = 0x65;
const ZERO: u8 = 0x30;
const NINE: u8 = 0x39;
const COLON: u8 = 0x3A;

/// Struct that is in charge of reading a given .torrent file and then creating a
/// corresponding `Torrent` struct that represents the file contents.

pub struct TorrentDecoder {
    bytes: Vec<u8>,
    pos: usize,
    info_pos: usize,
}

/// Represents every data type that a bencoded file supports (integers, strings, lists and dictionaries).

enum BType {
    Integer(i32),
    String(Vec<u8>),
    List(Vec<BType>),
    Dictionary(HashMap<String, BType>),
}

/// Given the content of a .torrent file, it creates a `TorrentDecoder` that is ready to decode it.
/// The file content is provided as a vector of bytes.

impl From<Vec<u8>> for TorrentDecoder {
    fn from(bytes: Vec<u8>) -> TorrentDecoder {
        TorrentDecoder {
            bytes,
            pos: 0_usize,
            info_pos: 0_usize,
        }
    }
}

impl TorrentDecoder {
    /// Given a .torrent file path, it reads and decodes its data, creating a valid `Torrent` struct (if possible).
    /// This `Torrent` struct stores the relevant information that a torrent file download needs to be possible.

    pub fn decode(path: &str) -> Result<Torrent, Error> {
        let bytes = TorrentDecoder::read_torrent_bytes(path)?;
        let mut decoder = TorrentDecoder::from(bytes);
        let btype = decoder.get_btype()?;
        decoder.get_torrent(btype)
    }

    /// Reads every byte of the file located at `path` and returns them as a vector.

    fn read_torrent_bytes(path: &str) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();
        File::open(Path::new(path))?.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Translates the loaded bytes in a `TorrentDecoder`, into a `BType`'d recursive structure and returns it.
    /// This structure is a 1:1 representation of what a bencoded file contains.

    fn get_btype(&mut self) -> Result<BType, Error> {
        let btype = self.decode_next()?;
        if !self.eof() {
            return self.error("missing tailing character");
        }
        Ok(btype)
    }

    /// Reads the current byte and builts a whole `BType`'d structure depending on what value was read.

    fn decode_next(&mut self) -> Result<BType, Error> {
        match self.bytes[self.pos] {
            INT_START => self.decode_integer(),
            ZERO..=NINE => self.decode_string(),
            LIST_START => self.decode_list(),
            DICT_START => self.decode_dictionary(),
            _ => self.error("invalid type identifier"),
        }
    }

    /// Builds a BType::Integer with bytes between "i" and "e" delimiters.

    fn decode_integer(&mut self) -> Result<BType, Error> {
        self.pos += 1;
        Ok(BType::Integer(self.decode_number(INT_END)?))
    }

    /// Reads every byte until a delimiter character `limit_char`, and makes a 32-bit integer value with them.

    fn decode_number(&mut self, limit_char: u8) -> Result<i32, Error> {
        match self.find(limit_char) {
            Some(limit_pos) => {
                let bytes = self.bytes[self.pos..limit_pos].to_vec();
                self.pos = limit_pos + 1;
                let int_as_string = self.bytes_to_string(&bytes)?;
                if self.is_invalid_int(&int_as_string) {
                    return self.error("invalid integer");
                }
                int_as_string
                    .parse::<i32>()
                    .map_err(|_err| Error::new(ErrorKind::Other, "integer parsing error"))
            }
            None => self.error("number delimiter not found"),
        }
    }

    /// Searches for a byte with `char` value, and returns its position (if found).

    fn find(&mut self, char: u8) -> Option<usize> {
        let mut start = self.pos;
        let end = self.bytes.len();

        while start < end {
            if self.bytes[start] == char {
                return Some(start);
            }
            start += 1
        }
        None
    }

    /// Transforms a collection of bytes into a valid UTF-8 string.

    fn bytes_to_string(&self, bytes: &[u8]) -> Result<String, Error> {
        String::from_utf8(bytes.to_owned())
            .map_err(|_err| Error::new(ErrorKind::Other, "UTF-8 encoding error"))
    }

    /// Returns true if the read integer value is an invalid bencoding.

    fn is_invalid_int(&self, int: &str) -> bool {
        int.starts_with("-0") || (int.starts_with('0') && (int.len() > 1))
    }

    /// Creates an instance of an `io::Error` with a specificied `msg`.

    fn error<T>(&self, msg: &str) -> Result<T, Error> {
        Err(Error::new(ErrorKind::Other, msg))
    }

    /// Builds a BType::String by reading `len` bytes, starting from the ":" delimiter.

    fn decode_string(&mut self) -> Result<BType, Error> {
        let len = self.decode_number(COLON)? as usize;
        let bytes = self.bytes[self.pos..(self.pos + len)].to_vec();
        self.pos += len;
        Ok(BType::String(bytes))
    }

    /// Builds a BType::List structure by decoding and building
    /// every other `BType`'d structure located between "l" and "e" delimiters.

    fn decode_list(&mut self) -> Result<BType, Error> {
        let mut list: Vec<BType> = Vec::new();
        self.pos += 1;

        while self.bytes[self.pos] != LIST_END {
            let value = self.decode_next()?;
            list.push(value);
        }
        self.pos += 1;
        Ok(BType::List(list))
    }

    /// Builds a BType::Dictionary structure with bytes between "d" and "e" delimiters.
    /// Every existing key must be converted into a valid BType::String.
    /// Associated values can be of any BType variant.

    fn decode_dictionary(&mut self) -> Result<BType, Error> {
        let mut dict: HashMap<String, BType> = HashMap::new();
        let mut key;
        self.pos += 1;

        while self.bytes[self.pos] != DICT_END {
            match self.decode_string()? {
                BType::String(bytes) => key = self.bytes_to_string(&bytes)?,
                _ => return self.error("dictionary key is not a string"),
            }

            if key == "info" {
                self.info_pos = self.pos;
            }

            let value = self.decode_next()?;
            dict.insert(key, value);
        }
        self.pos += 1;
        Ok(BType::Dictionary(dict))
    }

    /// Checks if the `TorrentDecoder` has already finished processing every loaded byte.

    fn eof(&self) -> bool {
        self.bytes.len() <= self.pos
    }

    /// Creates a valid `Torrent` struct out of the data given by a `BType`'d structure (if possible).
    /// To make this possible the `BType`'d structure needs to include every value that is essential
    /// to a torrent file download (e.g., "announce", "info", "piece length", "pieces", "name" and "files" or "length").

    fn get_torrent(&self, btype: BType) -> Result<Torrent, Error> {
        let file = match btype {
            BType::Dictionary(file) => file,
            _ => return self.error("decoded file is not a bencoded dictionary"),
        };

        let info = match file.get("info") {
            Some(BType::Dictionary(info)) => info,
            _ => return self.error("info key not present or has invalid value type"),
        };

        let announce = self.get_string_from(&file, "announce")?;
        let name = self.get_string_from(info, "name")?;
        let piece_length = self.get_integer_from(info, "piece length")?;

        let pieces = match info.get("pieces") {
            Some(BType::String(bytes)) => self.valid_pieces(bytes)?,
            _ => return self.error("pieces key not present or has invalid value type"),
        };

        let files = match (info.get("length"), info.get("files")) {
            (Some(BType::Integer(length)), None) => self.single_file_list(name, *length),
            (None, Some(BType::List(list))) => self.multiple_file_list(name, list)?,
            _ => return self.error("length and files keys not present or have invalid types"),
        };

        self.create_torrent(announce, piece_length, pieces, files, self.get_info_hash()?)
    }

    /// Looks up a `BType`'d string value associated to a `key`, in a given `dict`.
    /// If a string value is found for `key`, it is then converted from its "raw" state (bytes) to a UTF-8 valid format.

    fn get_string_from(&self, dict: &HashMap<String, BType>, key: &str) -> Result<String, Error> {
        let value = match dict.get(key) {
            Some(BType::String(bytes)) => self.bytes_to_string(bytes)?,
            _ => return self.error(&format!("{} key not present or has invalid type", key)),
        };
        Ok(value)
    }

    /// Looks up a `BType`'d integer value associated to a `key`, in a given `dict`.

    fn get_integer_from(&self, dict: &HashMap<String, BType>, key: &str) -> Result<i32, Error> {
        let value = match dict.get(key) {
            Some(BType::Integer(int)) => *int,
            _ => return self.error(&format!("{} key not present or has invalid type", key)),
        };
        Ok(value)
    }

    /// Checks if the pieces key bytes are multiple of 20.

    fn valid_pieces(&self, pieces: &[u8]) -> Result<Vec<u8>, Error> {
        if pieces.len() % 20 == 0 {
            return Ok(pieces.to_owned());
        }
        self.error("pieces string is not a multiple of 20")
    }

    /// Given a `name` and `length` for a file, it creates a vector of only one `SingleFile` struct.

    fn single_file_list(&self, name: String, length: i32) -> Vec<SingleFile> {
        vec![SingleFile { length, path: name }]
    }

    /// Given a `name` and a vector of `BType`'d dictionaries, it creates a vector of valid `SingleFile` structs.

    fn multiple_file_list(
        &self,
        name: String,
        btype_list: &[BType],
    ) -> Result<Vec<SingleFile>, Error> {
        let mut file_list: Vec<SingleFile> = Vec::with_capacity(btype_list.len());
        if btype_list.is_empty() {
            return self.error("multiple file torrent does not have any files");
        }

        for btype in btype_list {
            let single_file = match btype {
                BType::Dictionary(file_dict) => self.single_file_from_dict(&name, file_dict)?,
                _ => return self.error("some file of the list is not a bencoded dictionary"),
            };
            file_list.push(single_file);
        }
        Ok(file_list)
    }

    /// Creates a `SingleFile` struct from the values `length` and `path` that the passed dictionary has.
    /// The `directory_name` parameter is used to set it as a prefix for the path field of the `SingleFile`.

    fn single_file_from_dict(
        &self,
        directory_name: &str,
        file_dict: &HashMap<String, BType>,
    ) -> Result<SingleFile, Error> {
        let single_file = match (file_dict.get("length"), file_dict.get("path")) {
            (Some(BType::Integer(length)), Some(BType::List(path))) => SingleFile {
                length: *length,
                path: self.get_complete_path(directory_name, path)?,
            },
            _ => return self.error("missing keys in single file dictionary"),
        };
        Ok(single_file)
    }

    /// Given a `directory_name` and a vector of `BTyped`'d strings, it creates a valid UTF-8 `String`
    /// that represents the whole path to a file.
    /// `directory_name` is used as the root directory by putting it at the beginning of the path.

    fn get_complete_path(
        &self,
        directory_name: &str,
        btype_path: &[BType],
    ) -> Result<String, Error> {
        if btype_path.is_empty() {
            return self.error("missing name in single file");
        }

        let mut complete_path = directory_name.to_string();
        for btype in btype_path {
            match btype {
                BType::String(bytes) => {
                    let sub_directory = self.bytes_to_string(bytes)?;
                    complete_path.push('/');
                    complete_path.push_str(&sub_directory)
                }
                _ => return self.error("element in path list is not a string"),
            }
        }
        Ok(complete_path)
    }

    /// Returns a 20-byte array containing the result of applying SHA1 hash algorithm
    /// to the info dictionary raw content (bytes).

    fn get_info_hash(&self) -> Result<[u8; 20], Error> {
        let info_bytes = self.bytes[self.info_pos..self.bytes.len()].to_vec();
        let hasher = Sha1::new_with_prefix(info_bytes);
        hasher
            .finalize()
            .to_vec()
            .try_into()
            .map_err(|_err| Error::new(ErrorKind::Other, "info hash creation error"))
    }

    /// Creates a new instance of a `Torrent` struct with the given values.

    fn create_torrent(
        &self,
        announce: String,
        piece_length: i32,
        pieces: Vec<u8>,
        files: Vec<SingleFile>,
        info_hash: [u8; 20],
    ) -> Result<Torrent, Error> {
        Ok(Torrent {
            announce,
            piece_length,
            pieces,
            files,
            info_hash,
            tracker_info: TrackerInfo {
                interval: -1,
                complete: -1,
                incomplete: -1,
                tracker_id: String::new(),
                peers: Vec::new(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_integer() -> Result<(), Error> {
        let input_bytes = "i666e".as_bytes().to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        match d.get_btype()? {
            BType::Integer(integer) => assert_eq!(integer, 666),
            _ => return d.error("expected to decode a BType::Integer"),
        }
        Ok(())
    }

    #[test]
    fn test_negative_integer() -> Result<(), Error> {
        let input_bytes = "i-666e".as_bytes().to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        match d.get_btype()? {
            BType::Integer(integer) => assert_eq!(integer, -666),
            _ => return d.error("expected to decode a BType::Integer"),
        }
        Ok(())
    }

    #[test]
    fn test_negative_zero() {
        let input_bytes = "i-0e".as_bytes().to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        assert!(d.get_btype().is_err());
    }

    #[test]
    fn test_integer_with_leading_zero() {
        let input_bytes = "i03e".as_bytes().to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        assert!(d.get_btype().is_err());
    }

    #[test]
    fn test_string() -> Result<(), Error> {
        let input_bytes = b"21:Hola, me llamo Carlos".to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        match d.get_btype()? {
            BType::String(bytes) => assert_eq!(bytes, b"Hola, me llamo Carlos"),
            _ => return d.error("expected to decode a BType::String"),
        }
        Ok(())
    }

    #[test]
    fn test_empty_string() -> Result<(), Error> {
        let input_bytes = b"0:".to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        match d.get_btype()? {
            BType::String(bytes) => assert_eq!(bytes, b""),
            _ => return d.error("expected to decode a BType::String"),
        }
        Ok(())
    }

    #[test]
    fn test_empty_list() -> Result<(), Error> {
        let input_bytes = b"le".to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        match d.get_btype()? {
            BType::List(list) => assert_eq!(list.len(), 0),
            _ => return d.error("expected to decode a BType::List"),
        }
        Ok(())
    }

    #[test]
    fn test_empty_dictionary() -> Result<(), Error> {
        let input_bytes = b"de".to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        match d.get_btype()? {
            BType::Dictionary(dict) => assert!(dict.is_empty()),
            _ => return d.error("expected to decode a BType::Dictionary"),
        }
        Ok(())
    }

    #[test]
    fn test_dictionary() -> Result<(), Error> {
        let input_bytes = b"d4:spaml1:a1:bee".to_vec();

        let mut d = TorrentDecoder::from(input_bytes);
        match d.get_btype()? {
            BType::Dictionary(dict) => match dict.get("spam") {
                Some(BType::List(list)) => match (&list[0], &list[1]) {
                    (BType::String(a), BType::String(b)) => {
                        assert_eq!(a, b"a");
                        assert_eq!(b, b"b")
                    }
                    _ => return d.error("expected BType::String at 1st position in the list"),
                },
                _ => return d.error("expected to find spam key in dictionary"),
            },
            _ => return d.error("expected to decode a BType::Dictionary"),
        }
        Ok(())
    }

    /// Asserts that the single file that was read in `pos` has same values as `path` and `length`.

    fn assert_expected_single_file(torrent: &Torrent, pos: i32, path: &str, length: i32) {
        assert_eq!(torrent.files[pos as usize].path, path);
        assert_eq!(torrent.files[pos as usize].length, length);
    }

    #[test]
    fn test_single_file_torrent() -> Result<(), Error> {
        let file_bytes = TorrentDecoder::read_torrent_bytes("tests/sample.torrent")?;
        let mut aux = TorrentDecoder::from(file_bytes.clone());
        aux.info_pos = 83;
        let expected_info_hash = aux.get_info_hash()?;
        let expected_pieces = file_bytes[148..168].to_vec();

        let torrent = TorrentDecoder::decode("tests/sample.torrent")?;

        assert_eq!(torrent.announce, "udp://tracker.openbittorrent.com:80");
        assert_eq!(torrent.piece_length, 65536);
        assert_eq!(torrent.pieces, expected_pieces);
        assert_eq!(torrent.info_hash, expected_info_hash);
        assert_expected_single_file(&torrent, 0, "sample.txt", 20);
        Ok(())
    }

    #[test]
    fn test_multiple_file_torrent() -> Result<(), Error> {
        let file_bytes = TorrentDecoder::read_torrent_bytes("tests/bla.torrent")?;
        let mut aux = TorrentDecoder::from(file_bytes.clone());
        aux.info_pos = 176;
        let expected_info_hash = aux.get_info_hash()?;
        let expected_pieces = file_bytes[392..412].to_vec();

        let torrent = TorrentDecoder::decode("tests/bla.torrent")?;

        assert_eq!(
            torrent.announce,
            "udp://tracker.opentrackr.org:1337/announce"
        );
        assert_eq!(torrent.piece_length, 16384);
        assert_eq!(torrent.pieces, expected_pieces);
        assert_eq!(torrent.info_hash, expected_info_hash);
        assert_expected_single_file(&torrent, 0, "bla/sub_bla/a.txt", 8);
        assert_expected_single_file(&torrent, 1, "bla/sub_bla/neo_bla/b.txt", 8);
        assert_expected_single_file(&torrent, 2, "bla/main.rs", 5874);
        assert_expected_single_file(&torrent, 3, "bla/ideas_issues_tp.txt", 1294);
        Ok(())
    }
}
