use super::{
    super::{
        bdecoder::{BDecoder, BType},
        utils,
    },
    single_file::SingleFile,
    torrent::Torrent,
};
use crate::client::piece::Piece;
use std::{array::TryFromSliceError, collections::HashMap, vec};

pub fn from_bytes(bytes: Vec<u8>) -> Result<Torrent, String> {
    let file = match BDecoder::bdecode(bytes.clone())? {
        BType::Dictionary(file) => file,
        _ => return Err("decoded file is not a bencoded dictionary".to_string()),
    };

    let info = match file.get("info") {
        Some(BType::Dictionary(info)) => info,
        _ => return Err("info key not present or has invalid value type".to_string()),
    };

    let announce = get_string_from(&file, "announce")?;
    let name = get_string_from(info, "name")?;
    let piece_length = get_integer_from(info, "piece length")? as usize;

    let pieces = match info.get("pieces") {
        Some(BType::String(bytes)) => torrent_pieces_list(bytes, piece_length as usize)?,
        _ => return Err("pieces key not present or has invalid value type".to_string()),
    };

    let files = match (info.get("length"), info.get("files")) {
        (Some(BType::Integer(length)), None) => single_file_list(name.clone(), *length),
        (None, Some(BType::List(list))) => multiple_file_list(name.clone(), list)?,
        _ => return Err("length and files keys not present or have invalid types".to_string()),
    };

    Torrent::new(
        utils::remove_extension(&name),
        announce,
        pieces,
        files,
        get_info_hash(bytes)?,
    )
}

/// Looks up a `BType`'d string value associated to a `key`, in a given `dict`.
/// If a string value is found for `key`, it is then converted from its "raw" state (bytes) to a UTF-8 valid format.
fn get_string_from(dict: &HashMap<String, BType>, key: &str) -> Result<String, String> {
    let value = match dict.get(key) {
        Some(BType::String(bytes)) => utils::bytes_to_string(bytes)?,
        _ => return Err(format!("{} key not present or has invalid type", key)),
    };
    Ok(value)
}

/// Looks up a `BType`'d integer value associated to a `key`, in a given `dict`.
fn get_integer_from(dict: &HashMap<String, BType>, key: &str) -> Result<i64, String> {
    let value = match dict.get(key) {
        Some(BType::Integer(int)) => *int,
        _ => return Err(format!("{} key not present or has invalid type", key)),
    };
    Ok(value)
}

/// Checks if the pieces key bytes are multiple of 20.
fn torrent_pieces_list(pieces: &[u8], piece_length: usize) -> Result<Vec<Piece>, String> {
    if pieces.len() % 20 != 0 {
        return Err("pieces string is not a multiple of 20".to_string());
    }

    let mut final_pieces = Vec::with_capacity(pieces.len() / 20);
    for piece in pieces.chunks_exact(20).enumerate() {
        final_pieces.push(Piece::new(
            piece.0,
            piece_length,
            piece
                .1
                .try_into()
                .map_err(|e: TryFromSliceError| e.to_string())?,
        ));
    }
    Ok(final_pieces)
}

/// Given a `name` and `length` for a file, it creates a vector of only one `SingleFile` struct.
fn single_file_list(name: String, length: i64) -> Vec<SingleFile> {
    vec![SingleFile::new(length, name)]
}

/// Given a `name` and a vector of `BType`'d dictionaries, it creates a vector of valid `SingleFile` structs.
fn multiple_file_list(name: String, btype_list: &[BType]) -> Result<Vec<SingleFile>, String> {
    let mut file_list: Vec<SingleFile> = Vec::with_capacity(btype_list.len());
    if btype_list.is_empty() {
        return Err("multiple file torrent does not have any files".to_string());
    }

    for btype in btype_list {
        let single_file = match btype {
            BType::Dictionary(file_dict) => single_file_from_dict(&name, file_dict)?,
            _ => return Err("some file of the list is not a bencoded dictionary".to_string()),
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
) -> Result<SingleFile, String> {
    let single_file = match (file_dict.get("length"), file_dict.get("path")) {
        (Some(BType::Integer(length)), Some(BType::List(path))) => SingleFile {
            length: *length,
            path: get_complete_path(directory_name, path)?,
        },
        _ => return Err("missing keys in single file dictionary".to_string()),
    };
    Ok(single_file)
}

/// Given a `directory_name` and a vector of `BTyped`'d strings, it creates a valid UTF-8 `String`
/// that represents the whole path to a file.
/// `directory_name` is used as the root directory by putting it at the beginning of the path.
fn get_complete_path(directory_name: &str, btype_path: &[BType]) -> Result<String, String> {
    if btype_path.is_empty() {
        return Err("missing name in single file".to_string());
    }

    let mut complete_path = directory_name.to_string();
    for btype in btype_path {
        match btype {
            BType::String(bytes) => {
                let sub_directory = utils::bytes_to_string(bytes)?;
                complete_path.push('/');
                complete_path.push_str(&sub_directory)
            }
            _ => return Err("element in path list is not a string".to_string()),
        }
    }
    Ok(complete_path)
}

fn get_info_hash(bytes: Vec<u8>) -> Result<[u8; 20], String> {
    let info_start = match bytes.windows(7).position(|bytes| bytes == b"4:infod") {
        Some(info_key) => info_key + 6,
        None => return Err("info_key not present".to_string()),
    };
    utils::sha1(&bytes[info_start..bytes.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_single_file_torrent() -> Result<(), String> {
        let file_bytes = fs::read("tests/sample.torrent").map_err(|e| e.to_string())?;

        let expected_torrent = Torrent::new(
            "sample".to_string(),
            "udp://tracker.openbittorrent.com:80".to_string(),
            torrent_pieces_list(&file_bytes[148..168].to_vec(), 65536)?,
            vec![SingleFile::new(20, "sample.txt".to_string())],
            [
                0xd0, 0xd1, 0x4c, 0x92, 0x6e, 0x6e, 0x99, 0x76, 0x1a, 0x2f, 0xdc, 0xff, 0x27, 0xb4,
                0x03, 0xd9, 0x63, 0x76, 0xef, 0xf6,
            ],
        )?;

        let torrent = from_bytes(file_bytes)?;
        assert_eq!(expected_torrent, torrent);
        Ok(())
    }

    #[test]
    fn test_multiple_file_torrent() -> Result<(), String> {
        let file_bytes = fs::read("tests/bla.torrent").map_err(|e| e.to_string())?;

        let expected_torrent = Torrent::new(
            "bla".to_string(),
            "udp://tracker.opentrackr.org:1337/announce".to_string(),
            torrent_pieces_list(&file_bytes[392..412].to_vec(), 16384)?,
            vec![
                SingleFile::new(8, "bla/sub_bla/a.txt".to_string()),
                SingleFile::new(8, "bla/sub_bla/neo_bla/b.txt".to_string()),
                SingleFile::new(5874, "bla/main.rs".to_string()),
                SingleFile::new(1294, "bla/ideas_issues_tp.txt".to_string()),
            ],
            [
                0x89, 0xae, 0x9d, 0x0b, 0xe3, 0x7b, 0xed, 0xbc, 0x97, 0x66, 0xee, 0x85, 0x11, 0x91,
                0x16, 0xe9, 0x4f, 0x40, 0xc8, 0xe5,
            ],
        )?;

        let torrent = from_bytes(file_bytes)?;
        assert_eq!(expected_torrent, torrent);
        Ok(())
    }
}
