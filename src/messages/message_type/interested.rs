use std::io::Write;
use std::net::TcpStream;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Interested {
    len: u32,
    id: u8,
}

impl Interested {
    pub fn new() -> Interested {
        Interested { len: 1, id: 2 }
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&u32::to_be_bytes(self.len))?;
        stream.write_all(&[self.id])?;
        Ok(())
    }
}
