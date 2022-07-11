use std::{io::Write, net::TcpStream};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Cancel {
    len: u32,
    id: u8,
    index: u32,
    begin: u32,
    length: u32,
}

impl Cancel {
    pub fn new(index: u32, begin: u32, length: u32) -> Cancel {
        Cancel {
            len: 13,
            id: 8,
            index,
            begin,
            length: length as u32,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let size = self.length as i32 - self.begin as i32;
        if size <= 16384 {
            stream.write_all(&self.len.to_be_bytes())?;
            stream.write_all(&self.id.to_be_bytes())?;
            stream.write_all(&self.index.to_be_bytes())?;
            stream.write_all(&self.begin.to_be_bytes())?;
            stream.write_all(&self.length.to_be_bytes())?;
        }
        Ok(())
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn get_begin(&self) -> u32 {
        self.begin
    }
}
