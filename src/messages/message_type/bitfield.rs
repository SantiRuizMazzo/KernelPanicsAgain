#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Bitfield {
    // 4 byte
    len: u32,
    // 1 byte
    id: u8,
    // uso un vec en lugar del [a,b] que veniamos usando porque no puedo saber el largo del
    //bitfield en tiempo de compilacion
    bitfield: Vec<u8>,
}

impl Bitfield {
    pub fn new(len: u32, bitfield: Vec<u8>) -> Bitfield {
        Bitfield {
            len,
            id: 5,
            bitfield,
        }
    }

    /*pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&self.len.to_be_bytes())?;
        stream.write_all(&[self.id])?;
        stream.write_all(&self.bitfield)?;
        Ok(())
    }*/
}
