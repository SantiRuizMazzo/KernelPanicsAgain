use std::{
    convert::Infallible,
    fs::File,
    io::{Error, Read, Write},
};

use sha1::{Digest, Sha1};

/// Returns a 20-byte array containing the result of applying SHA1 hash algorithm to the given collection of bytes.
pub fn sha1(bytes: impl AsRef<[u8]>) -> Result<[u8; 20], String> {
    let hasher = Sha1::new_with_prefix(bytes);
    hasher
        .finalize()
        .try_into()
        .map_err(|e: Infallible| e.to_string())
}

/// Transforms a collection of bytes into a valid UTF-8 string.
pub fn bytes_to_string(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_owned()).map_err(|e| e.to_string())
}

pub fn round_up(base: usize, multiple: usize) -> usize {
    ((base + (multiple - 1)) / multiple) * multiple
}

pub fn remove_extension(file_name: &str) -> String {
    let mut split: Vec<&str> = file_name.split('.').collect();
    if split.len() > 1 {
        split.pop();
    }
    split.join("")
}

pub fn read_piece_file(download_path: String, piece_index: usize) -> Result<Vec<u8>, String> {
    let path = format!("{download_path}/{piece_index}");
    let err = |e: Error| e.to_string();
    let mut file = File::open(path).map_err(err)?;
    let mut read_bytes = Vec::new();
    file.read_to_end(&mut read_bytes).map_err(err)?;
    Ok(read_bytes)
}

pub fn append_to_file(file: &mut File, bytes_to_append: Vec<u8>) -> Result<(), String> {
    file.write_all(&bytes_to_append).map_err(|e| e.to_string())
}
