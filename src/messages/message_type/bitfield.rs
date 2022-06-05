//use std::io::Write;
//use std::net::TcpStream;
/*
pub struct Bitfield<'a> {
    // 4 byte
    len: u8,
    // 1 byte
    id: &'a [u8],
    // uso un vec en lugar del [a,b] que veniamos usando porque no puedo saber el largo del
    //bitfield en tiempo de compilacion
    bitfield: Vec<u8>,
}

impl Bitfield<'_> {
    pub fn new(len: u8, bitfield: Vec<u8>) -> Bitfield<'static> {
        Bitfield {
            len,
            id: &[5],
            bitfield,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(&[self.len])?;
        stream.write_all(self.id)?;
        stream.write_all(&self.bitfield)?;
        Ok(())
    }

}
*/
