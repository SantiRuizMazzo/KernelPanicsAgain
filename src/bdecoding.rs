use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

const INT_START: u8 = 0x69;
const INT_END: u8 = 0x65;
const LIST_START: u8 = 0x6C;
const LIST_END: u8 = 0x65;
const DICT_START: u8 = 0x64;
const DICT_END: u8 = 0x65;
const ZERO: u8 = 0x30;
const NINE: u8 = 0x39;
const COLON: u8 = 0x3A;

///

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
    ///

    fn new(bytes: Vec<u8>) -> BDecoder {
        BDecoder {
            bytes,
            pos: 0_usize,
        }
    }

    ///

    pub fn bdecode(bytes: Vec<u8>) -> Result<BType, Error> {
        let mut bdecoder = BDecoder::new(bytes);
        let btype_structure = bdecoder.decode_next()?;
        if !bdecoder.eof() {
            return bdecoder.error("missing tailing character");
        }
        Ok(btype_structure)
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

    fn decode_number(&mut self, limit_char: u8) -> Result<i64, Error> {
        match self.find(limit_char) {
            Some(limit_pos) => {
                let bytes = self.bytes[self.pos..limit_pos].to_vec();
                self.pos = limit_pos + 1;
                let int_as_string = self.bytes_to_string(&bytes)?;
                if self.is_invalid_int(&int_as_string) {
                    return self.error("invalid integer");
                }
                int_as_string
                    .parse::<i64>()
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
    fn test_positive_integer() -> Result<(), Error> {
        let input_bytes = "i666e".as_bytes().to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Integer(integer) => assert_eq!(integer, 666),
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "expected to decode a BType::Integer",
                ))
            }
        }
        Ok(())
    }

    #[test]
    fn test_negative_integer() -> Result<(), Error> {
        let input_bytes = "i-666e".as_bytes().to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Integer(integer) => assert_eq!(integer, -666),
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "expected to decode a BType::Integer",
                ))
            }
        }
        Ok(())
    }

    #[test]
    fn test_negative_zero() {
        let input_bytes = "i-0e".as_bytes().to_vec();

        assert!(BDecoder::bdecode(input_bytes).is_err());
    }

    #[test]
    fn test_integer_with_leading_zero() {
        let input_bytes = "i03e".as_bytes().to_vec();

        assert!(BDecoder::bdecode(input_bytes).is_err());
    }

    #[test]
    fn test_string() -> Result<(), Error> {
        let input_bytes = b"21:Hola, me llamo Carlos".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::String(bytes) => assert_eq!(bytes, b"Hola, me llamo Carlos"),
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "expected to decode a BType::String",
                ))
            }
        }
        Ok(())
    }

    #[test]
    fn test_empty_string() -> Result<(), Error> {
        let input_bytes = b"0:".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::String(bytes) => assert_eq!(bytes, b""),
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "expected to decode a BType::String",
                ))
            }
        }
        Ok(())
    }

    #[test]
    fn test_empty_list() -> Result<(), Error> {
        let input_bytes = b"le".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::List(list) => assert_eq!(list.len(), 0),
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "expected to decode a BType::List",
                ))
            }
        }
        Ok(())
    }

    #[test]
    fn test_empty_dictionary() -> Result<(), Error> {
        let input_bytes = b"de".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Dictionary(dict) => assert!(dict.is_empty()),
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "expected to decode a BType::Dictionary",
                ))
            }
        }
        Ok(())
    }

    #[test]
    fn test_dictionary() -> Result<(), Error> {
        let input_bytes = b"d4:spaml1:a1:bee".to_vec();

        match BDecoder::bdecode(input_bytes)? {
            BType::Dictionary(dict) => match dict.get("spam") {
                Some(BType::List(list)) => match (&list[0], &list[1]) {
                    (BType::String(a), BType::String(b)) => {
                        assert_eq!(a, b"a");
                        assert_eq!(b, b"b")
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "expected BType::String at 1st position in the list",
                        ))
                    }
                },
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "expected to find spam key in dictionary",
                    ))
                }
            },
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "expected to decode a BType::Dictionary",
                ))
            }
        }
        Ok(())
    }
}
