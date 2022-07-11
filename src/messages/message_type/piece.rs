use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::TcpStream;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Piece {
    pub index: u32,
    pub begin: u32,
    id: u8,
    pub block: Vec<u8>,
}

impl Piece {
    pub fn new(index: u32, begin: u32, block: Vec<u8>) -> Piece {
        Piece {
            id: 7,
            index,
            begin,
            block,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let total_message_length = 9 + self.block.len() as u32;

        stream.write_all(&u32::to_be_bytes(total_message_length))?;
        stream.write_all(&[self.id])?;
        stream.write_all(&u32::to_be_bytes(self.index))?;
        stream.write_all(&u32::to_be_bytes(self.begin))?;
        stream.write_all(&self.block)?;

        Ok(())
    }

    pub fn load_block(&mut self, torrent_path: String) -> Result<(), String> {
        let piece_path = format!("{torrent_path}/.tmp/{}", self.index);
        let mut file = File::open(piece_path).map_err(|err| err.to_string())?;
        file.seek(SeekFrom::Start(self.begin as u64))
            .map_err(|err| err.to_string())?;
        file.read_exact(&mut self.block)
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}
