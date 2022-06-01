use super::{
    super::{
        bdecoding::{BDecoder, BType},
        utils::{bytes_to_string, sha1},
    },
    single_file::SingleFile,
    torrent::Torrent,
};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    vec,
};

///

pub fn torrent_from_bytes(bytes: Vec<u8>) -> Result<Torrent, Error> {
    let file = match BDecoder::bdecode(bytes.clone())? {
        BType::Dictionary(file) => file,
        _ => return error("decoded file is not a bencoded dictionary"),
    };

    let info = match file.get("info") {
        Some(BType::Dictionary(info)) => info,
        _ => return error("info key not present or has invalid value type"),
    };

    let announce = get_string_from(&file, "announce")?;
    let name = get_string_from(info, "name")?;
    let piece_length = get_integer_from(info, "piece length")?;

    let pieces = match info.get("pieces") {
        Some(BType::String(bytes)) => valid_pieces(bytes)?,
        _ => return error("pieces key not present or has invalid value type"),
    };

    let files = match (info.get("length"), info.get("files")) {
        (Some(BType::Integer(length)), None) => single_file_list(name, *length),
        (None, Some(BType::List(list))) => multiple_file_list(name, list)?,
        _ => return error("length and files keys not present or have invalid types"),
    };

    Ok(Torrent::new(
        announce,
        piece_length,
        pieces,
        files,
        get_info_hash(bytes)?,
    ))
}

/// Creates an instance of an `io::Error` with a specificied `msg`.

fn error<T>(msg: &str) -> Result<T, Error> {
    Err(Error::new(ErrorKind::Other, msg))
}

/// Looks up a `BType`'d string value associated to a `key`, in a given `dict`.
/// If a string value is found for `key`, it is then converted from its "raw" state (bytes) to a UTF-8 valid format.

fn get_string_from(dict: &HashMap<String, BType>, key: &str) -> Result<String, Error> {
    let value = match dict.get(key) {
        Some(BType::String(bytes)) => bytes_to_string(bytes)?,
        _ => return error(&format!("{} key not present or has invalid type", key)),
    };
    Ok(value)
}

/// Looks up a `BType`'d integer value associated to a `key`, in a given `dict`.

fn get_integer_from(dict: &HashMap<String, BType>, key: &str) -> Result<i64, Error> {
    let value = match dict.get(key) {
        Some(BType::Integer(int)) => *int,
        _ => return error(&format!("{} key not present or has invalid type", key)),
    };
    Ok(value)
}

/// Checks if the pieces key bytes are multiple of 20.

fn valid_pieces(pieces: &[u8]) -> Result<Vec<u8>, Error> {
    if pieces.len() % 20 == 0 {
        return Ok(pieces.to_owned());
    }
    error("pieces string is not a multiple of 20")
}

/// Given a `name` and `length` for a file, it creates a vector of only one `SingleFile` struct.

fn single_file_list(name: String, length: i64) -> Vec<SingleFile> {
    vec![SingleFile { length, path: name }]
}

/// Given a `name` and a vector of `BType`'d dictionaries, it creates a vector of valid `SingleFile` structs.

fn multiple_file_list(name: String, btype_list: &[BType]) -> Result<Vec<SingleFile>, Error> {
    let mut file_list: Vec<SingleFile> = Vec::with_capacity(btype_list.len());
    if btype_list.is_empty() {
        return error("multiple file torrent does not have any files");
    }

    for btype in btype_list {
        let single_file = match btype {
            BType::Dictionary(file_dict) => single_file_from_dict(&name, file_dict)?,
            _ => return error("some file of the list is not a bencoded dictionary"),
        };
        file_list.push(single_file);
    }
    Ok(file_list)
}

/// Creates a `SingleFile` struct from the values `length` and `path` that the passed dictionary has.
/// The `directory_name` parameter is used to set it as a prefix for the path field of the `SingleFile`.

fn single_file_from_dict(
    directory_name: &str,
    file_dict: &HashMap<String, BType>,
) -> Result<SingleFile, Error> {
    let single_file = match (file_dict.get("length"), file_dict.get("path")) {
        (Some(BType::Integer(length)), Some(BType::List(path))) => SingleFile {
            length: *length,
            path: get_complete_path(directory_name, path)?,
        },
        _ => return error("missing keys in single file dictionary"),
    };
    Ok(single_file)
}

/// Given a `directory_name` and a vector of `BTyped`'d strings, it creates a valid UTF-8 `String`
/// that represents the whole path to a file.
/// `directory_name` is used as the root directory by putting it at the beginning of the path.

fn get_complete_path(directory_name: &str, btype_path: &[BType]) -> Result<String, Error> {
    if btype_path.is_empty() {
        return error("missing name in single file");
    }

    let mut complete_path = directory_name.to_string();
    for btype in btype_path {
        match btype {
            BType::String(bytes) => {
                let sub_directory = bytes_to_string(bytes)?;
                complete_path.push('/');
                complete_path.push_str(&sub_directory)
            }
            _ => return error("element in path list is not a string"),
        }
    }
    Ok(complete_path)
}

///

fn get_info_hash(bytes: Vec<u8>) -> Result<[u8; 20], Error> {
    let info_start = match bytes.windows(7).position(|bytes| bytes == b"4:infod") {
        Some(info_key) => info_key + 6,
        None => return error("info_key not present"),
    };
    sha1(&bytes[info_start..bytes.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read;

    /// Asserts that the single file that was read in `pos` has same values as `path` and `length`.

    fn assert_expected_single_file(torrent: &Torrent, pos: i32, path: &str, length: i64) {
        assert_eq!(torrent.files[pos as usize].path, path);
        assert_eq!(torrent.files[pos as usize].length, length);
    }

    #[test]
    fn test_single_file_torrent() -> Result<(), Error> {
        let file_bytes = read("tests/sample.torrent")?;
        let expected_info_hash = [
            0xd0, 0xd1, 0x4c, 0x92, 0x6e, 0x6e, 0x99, 0x76, 0x1a, 0x2f, 0xdc, 0xff, 0x27, 0xb4,
            0x03, 0xd9, 0x63, 0x76, 0xef, 0xf6,
        ];
        let expected_pieces = file_bytes[148..168].to_vec();

        let torrent = torrent_from_bytes(file_bytes)?;

        assert_eq!(torrent.announce, "udp://tracker.openbittorrent.com:80");
        assert_eq!(torrent.piece_length, 65536);
        assert_eq!(torrent.pieces, expected_pieces);
        assert_eq!(torrent.info_hash, expected_info_hash);
        assert_expected_single_file(&torrent, 0, "sample.txt", 20);
        Ok(())
    }

    #[test]
    fn test_multiple_file_torrent() -> Result<(), Error> {
        let file_bytes = read("tests/bla.torrent")?;
        let expected_info_hash = [
            0x89, 0xae, 0x9d, 0x0b, 0xe3, 0x7b, 0xed, 0xbc, 0x97, 0x66, 0xee, 0x85, 0x11, 0x91,
            0x16, 0xe9, 0x4f, 0x40, 0xc8, 0xe5,
        ];
        let expected_pieces = file_bytes[392..412].to_vec();

        let torrent = torrent_from_bytes(file_bytes)?;

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
