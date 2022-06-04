//use std::io::Write;
//use std::net::TcpStream;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Have {
    // 4 byte
    len: u8,
    // 1 byte
    id: u8,
    // index del hasheado
    index: u8,
}

impl Have {
    pub fn new(piece_index: u8) -> Have {
        Have {
            len: 5,
            id: 4,
            index: piece_index,
        }
    }
    /*
    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(&[self.len])?;
        stream.write_all(&[self.id])?;
        stream.write_all(&[self.index])?;
        Ok(())
    }
    */
}
