use std::io::Write;
use std::net::TcpStream;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Have {
    // 4 byte
    len: u32,
    // 1 byte
    id: u8,
    // index del hasheado
    index: u32,
}

impl Have {
    pub fn new(piece_index: u32) -> Have {
        Have {
            len: 5,
            id: 4,
            index: piece_index,
        }
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }
    /*
        pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
            stream.write_all(&[0])?;
            stream.write_all(&[0])?;
            stream.write_all(&[0])?;
            stream.write_all(self.len)?;
            stream.write_all(&[self.id])?;
            stream.write_all(self.index)?;

            Ok(())
        }
    */

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&u32::to_be_bytes(self.len))?;
        stream.write_all(&[self.id])?;
        stream.write_all(&u32::to_be_bytes(self.index))?;

        Ok(())
    }
}
