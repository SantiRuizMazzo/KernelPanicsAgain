use std::fmt::{Error, Write};

const PERIOD: u8 = 0x2e;
const UNDERSCORE: u8 = 0x5f;
const HYPHEN: u8 = 0x2d;
const TILDE: u8 = 0x7e;
const ENCODING_CHAR: char = '%';

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

pub fn encode<T: AsRef<[u8]>>(data: T) -> Result<String, Error> {
    let mut encoded = String::with_capacity(data.as_ref().len() * 2);
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";

    for &byte in data.as_ref().iter() {
        if is_url_valid(byte) {
            encoded.write_char(byte as char)?;
            continue;
        }
        encoded.write_char(ENCODING_CHAR)?;
        encoded.write_char(HEX_CHARS[(byte >> 4) as usize] as char)?;
        encoded.write_char(HEX_CHARS[(byte & 0xf) as usize] as char)?;
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencoding_ascii_string() -> Result<(), Error> {
        assert_eq!(
            "%3F%20and%20the%20Mysterians",
            encode("? and the Mysterians")?
        );
        Ok(())
    }

    #[test]
    fn urlencoding_utf8_string() -> Result<(), Error> {
        assert_eq!(
            "%E4%B8%8A%E6%B5%B7%2B%E4%B8%AD%E5%9C%8B",
            encode("上海+中國")?
        );
        Ok(())
    }

    #[test]
    fn urlencoding_byte_string() -> Result<(), Error> {
        assert_eq!(
            "%124Vx%9A%BC%DE%F1%23Eg%89%AB%CD%EF%124Vx%9A",
            encode(
                b"\x12\x34\x56\x78\x9a\xbc\xde\xf1\x23\x45\x67\x89\xab\xcd\xef\x12\x34\x56\x78\x9a"
            )?
        );
        Ok(())
    }
}
