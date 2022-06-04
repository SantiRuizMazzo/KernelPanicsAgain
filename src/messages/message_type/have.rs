/*
use std::io::Write;
use std::net::TcpStream;

pub struct Have<'a> {
    // 4 byte
    len: &'a [u8],
    // 1 byte
    id: &'a [u8],
    // index del hasheado
    index: &'a [u8],
}

impl Have<'_> {
    pub fn new(piece_index: &'static [u8]) -> Have<'static> {
        Have {
            len: &[5],
            id: &[4],
            index: piece_index,
        }
    }


    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(self.len)?;
        stream.write_all(self.id)?;
        stream.write_all(self.index)?;
        Ok(())
    }


}
*/
