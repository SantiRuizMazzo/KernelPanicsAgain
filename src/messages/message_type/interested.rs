//use std::io::Write;
//use std::net::TcpStream;
/*
pub struct Interested<'a> {
    // 4 byte
    len: &'a [u8],
    // 1 byte
    id: &'a [u8],
}

impl Interested<'_> {
    pub fn new() -> Interested<'static> {
        Interested {
            len: &[1],
            id: &[2],
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
