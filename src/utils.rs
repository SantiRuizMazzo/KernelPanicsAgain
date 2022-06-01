use sha1::{Digest, Sha1};
use std::io::{Error, ErrorKind};

/// Returns a 20-byte array containing the result of applying SHA1 hash algorithm to the given collection of bytes.

pub fn sha1(bytes: impl AsRef<[u8]>) -> Result<[u8; 20], Error> {
    let hasher = Sha1::new_with_prefix(bytes);
    hasher
        .finalize()
        .to_vec()
        .try_into()
        .map_err(|_err| Error::new(ErrorKind::Other, "hashing error"))
}

/// Transforms a collection of bytes into a valid UTF-8 string.

pub fn bytes_to_string(bytes: &[u8]) -> Result<String, Error> {
    String::from_utf8(bytes.to_owned())
        .map_err(|_err| Error::new(ErrorKind::Other, "UTF-8 encoding error"))
}
