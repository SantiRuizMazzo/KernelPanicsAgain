use crate::utils;
use std::collections::HashMap;

const INT_START: u8 = 0x69;
const INT_END: u8 = 0x65;
const LIST_START: u8 = 0x6C;
const LIST_END: u8 = 0x65;
const DICT_START: u8 = 0x64;
const DICT_END: u8 = 0x65;
const ZERO: u8 = 0x30;
const NINE: u8 = 0x39;
const COLON: u8 = 0x3A;

pub struct BDecoder {
    bytes: Vec<u8>,
    pos: usize,
}

/// Represents every data type that a bencoded file supports (integers, strings, lists and dictionaries).
pub enum BType {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<BType>),
    Dictionary(HashMap<String, BType>),
}

impl BDecoder {
    fn new(bytes: Vec<u8>) -> BDecoder {
        BDecoder {
            bytes,
            pos: 0_usize,
        }
    }

    pub fn bdecode(bytes: Vec<u8>) -> Result<BType, String> {
        let mut bdecoder = BDecoder::new(bytes);
        let btype_structure = bdecoder.decode_next()?;
        if !bdecoder.eof() {
            return Err("missing tailing character".to_string());
        }
        Ok(btype_structure)
    }

    /// Reads the current byte and builts a whole `BType`'d structure depending on what value was read.
    fn decode_next(&mut self) -> Result<BType, String> {
        match self.bytes[self.pos] {
            INT_START => self.decode_integer(),
            ZERO..=NINE => self.decode_string(),
            LIST_START => self.decode_list(),
            DICT_START => self.decode_dictionary(),
            _ => Err("invalid type identifier".to_string()),
        }
    }

    /// Builds a BType::Integer with bytes between "i" and "e" delimiters.
    fn decode_integer(&mut self) -> Result<BType, String> {
        self.pos += 1;
        Ok(BType::Integer(self.decode_number(INT_END)?))
    }

    /// Reads every byte until a delimiter character `limit_char`, and makes a 32-bit integer value with them.
    fn decode_number(&mut self, limit_char: u8) -> Result<i64, String> {
        match self.find(limit_char) {
            Some(limit_pos) => {
                let bytes = self.bytes[self.pos..limit_pos].to_vec();
                self.pos = limit_pos + 1;
                let int_as_string = utils::bytes_to_string(&bytes)?;
                if self.is_invalid_int(&int_as_string) {
                    return Err("invalid integer".to_string());
                }
                int_as_string.parse::<i64>().map_err(|err| err.to_string())
            }
            None => Err("number delimiter not found".to_string()),
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

    /// Returns true if the read integer value is an invalid bencoding.
    fn is_invalid_int(&self, int: &str) -> bool {
        int.starts_with("-0") || (int.starts_with('0') && (int.len() > 1))
    }

    /// Builds a BType::String by reading `len` bytes, starting from the ":" delimiter.
    fn decode_string(&mut self) -> Result<BType, String> {
        let len = self.decode_number(COLON)? as usize;
        let bytes = self.bytes[self.pos..(self.pos + len)].to_vec();
        self.pos += len;
        Ok(BType::String(bytes))
    }

    /// Builds a BType::List structure by decoding and building
    /// every other `BType`'d structure located between "l" and "e" delimiters.
    fn decode_list(&mut self) -> Result<BType, String> {
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
    fn decode_dictionary(&mut self) -> Result<BType, String> {
        let mut dict: HashMap<String, BType> = HashMap::new();
        let mut key;
        self.pos += 1;

        while self.bytes[self.pos] != DICT_END {
            match self.decode_string()? {
                BType::String(bytes) => key = utils::bytes_to_string(&bytes)?,
                _ => return Err("dictionary key is not a string".to_string()),
            }

            let value = self.decode_next()?;
            dict.insert(key, value);
        }
        self.pos += 1;
        Ok(BType::Dictionary(dict))
    }

    /// Checks if the `BDecoder` has already finished processing every loaded byte.
    fn eof(&self) -> bool {
        self.bytes.len() <= self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_integer() -> Result<(), String> {
        let input_bytes = b"i666e".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Integer(integer) => assert_eq!(integer, 666),
            _ => return Err("expected to decode a BType::Integer".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_negative_integer() -> Result<(), String> {
        let input_bytes = b"i-666e".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Integer(integer) => assert_eq!(integer, -666),
            _ => return Err("expected to decode a BType::Integer".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_negative_zero() {
        let input_bytes = b"i-0e".to_vec();
        assert!(BDecoder::bdecode(input_bytes).is_err());
    }

    #[test]
    fn test_integer_with_leading_zero() {
        let input_bytes = b"i03e".to_vec();
        assert!(BDecoder::bdecode(input_bytes).is_err());
    }

    #[test]
    fn test_string() -> Result<(), String> {
        let input_bytes = b"21:Hola, me llamo Carlos".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::String(bytes) => assert_eq!(bytes, b"Hola, me llamo Carlos"),
            _ => return Err("expected to decode a BType::String".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_empty_string() -> Result<(), String> {
        let input_bytes = b"0:".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::String(bytes) => assert_eq!(bytes, b""),
            _ => return Err("expected to decode a BType::String".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_empty_list() -> Result<(), String> {
        let input_bytes = b"le".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::List(list) => assert_eq!(list.len(), 0),
            _ => return Err("expected to decode a BType::List".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_empty_dictionary() -> Result<(), String> {
        let input_bytes = b"de".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Dictionary(dict) => assert!(dict.is_empty()),
            _ => return Err("expected to decode a BType::Dictionary".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_dictionary() -> Result<(), String> {
        let input_bytes = b"d4:spaml1:a1:bee".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Dictionary(dict) => match dict.get("spam") {
                Some(BType::List(list)) => match (&list[0], &list[1]) {
                    (BType::String(a), BType::String(b)) => {
                        assert_eq!(a, b"a");
                        assert_eq!(b, b"b")
                    }
                    _ => {
                        return Err("expected BType::String at 1st position in the list".to_string())
                    }
                },
                _ => return Err("expected to find spam key in dictionary".to_string()),
            },
            _ => return Err("expected to decode a BType::Dictionary".to_string()),
        }
        Ok(())
    }
}
