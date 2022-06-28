use sha1::{Digest, Sha1};

/// Returns a 20-byte array containing the result of applying SHA1 hash algorithm to the given collection of bytes.
pub fn sha1(bytes: impl AsRef<[u8]>) -> Result<[u8; 20], String> {
    let hasher = Sha1::new_with_prefix(bytes);
    hasher
        .finalize()
        .to_vec()
        .try_into()
        .map_err(|_| "hashing error".to_string())
}

/// Transforms a collection of bytes into a valid UTF-8 string.
pub fn bytes_to_string(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_owned()).map_err(|err| err.to_string())
}

pub fn round_up(base: usize, multiple: usize) -> usize {
    ((base + (multiple - 1)) / multiple) * multiple
}
