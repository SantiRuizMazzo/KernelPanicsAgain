use std::io::Write;
use std::net::TcpStream;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Request {
    len: u32,
    id: u8,
    pub index: u32,
    pub begin: u32,
    pub length: u32,
    sent: bool,
}

impl Request {
    pub fn new(index: u32, begin: u32, length: usize) -> Request {
        Request {
            len: 13,
            id: 6,
            index,
            begin,
            length: length as u32,
            sent: false,
        }
    }

    pub fn send(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        if self.sent {
            return Ok(());
        }

        let size = self.length as i32 - self.begin as i32;
        if size <= 16384 {
            stream.write_all(&self.len.to_be_bytes())?;
            stream.write_all(&self.id.to_be_bytes())?;
            stream.write_all(&self.index.to_be_bytes())?;
            stream.write_all(&self.begin.to_be_bytes())?;
            stream.write_all(&self.length.to_be_bytes())?;
        }
        self.sent = true;
        Ok(())
    }

    pub fn discarded(&mut self) {
        self.sent = false
    }
}
