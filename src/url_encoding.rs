use std::fmt::{Error, Write};

const PERIOD: u8 = 0x2e;
const UNDERSCORE: u8 = 0x5f;
const HYPHEN: u8 = 0x2d;
const TILDE: u8 = 0x7e;
const SPACE: u8 = 0x20;
const PLUS_SIGN: char = '+';
const ENCODING_CHAR: char = '%';
const HEX_CHARS: &[u8] = b"0123456789ABCDEF";

/// Returns true if character representation of `byte` is in the set 0-9, a-z, A-Z, '.', '-', '_' or '~'.
fn is_url_valid(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || byte == HYPHEN
        || byte == PERIOD
        || byte == UNDERSCORE
        || byte == TILDE
}

/// Receives a collection of bytes in `data` and returns a `String` where every URL-invalid byte from `data`
/// is replaced by "%nn", where nn is the hexadecimal representation of that byte value.
pub fn encode<T: AsRef<[u8]>>(data: T) -> Result<String, String> {
    let mut encoded = String::with_capacity(data.as_ref().len() * 2);
    let err = |e: Error| e.to_string();

    for &byte in data.as_ref() {
        if is_url_valid(byte) {
            encoded.write_char(byte as char).map_err(err)?;
            continue;
        } else if byte == SPACE {
            encoded.write_char(PLUS_SIGN).map_err(err)?;
            continue;
        }
        encoded.write_char(ENCODING_CHAR).map_err(err)?;
        encoded
            .write_char(HEX_CHARS[(byte >> 4) as usize] as char)
            .map_err(err)?;
        encoded
            .write_char(HEX_CHARS[(byte & 0xf) as usize] as char)
            .map_err(err)?;
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use crate::url_encoding;

    #[test]
    fn urlencoding_ascii_string() -> Result<(), String> {
        assert_eq!(
            "%3F+and+the+Mysterians",
            url_encoding::encode("? and the Mysterians")?
        );
        Ok(())
    }

    #[test]
    fn urlencoding_utf8_string() -> Result<(), String> {
        assert_eq!(
            "%E4%B8%8A%E6%B5%B7%2B%E4%B8%AD%E5%9C%8B",
            url_encoding::encode("上海+中國")?
        );
        Ok(())
    }

    #[test]
    fn urlencoding_byte_string() -> Result<(), String> {
        assert_eq!(
            "%124Vx%9A%BC%DE%F1%23Eg%89%AB%CD%EF%124Vx%9A",
            url_encoding::encode(
                b"\x12\x34\x56\x78\x9a\xbc\xde\xf1\x23\x45\x67\x89\xab\xcd\xef\x12\x34\x56\x78\x9a"
            )?
        );
        Ok(())
    }
}
