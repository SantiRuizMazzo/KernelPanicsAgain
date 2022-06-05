//use std::io::Write;
//use std::net::TcpStream;
/*
pub struct Unchoke<'a> {
    // 4 byte
    len: &'a [u8],
    // 1 byte
    id: &'a [u8],
}

impl Unchoke<'_> {
    pub fn new() -> Unchoke<'static> {
        Unchoke {
            len: &[1],
            id: &[1],
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(&[0])?;
        stream.write_all(self.len)?;
        stream.write_all(self.id)?;

        Ok(())
    }

}
*/
