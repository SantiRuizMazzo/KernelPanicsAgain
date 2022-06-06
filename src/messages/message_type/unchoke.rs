#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Unchoke {
    // 4 byte
    len: u32,
    // 1 byte
    id: u8,
}

impl Unchoke {
    pub fn new() -> Unchoke {
        Unchoke { len: 1, id: 1 }
    }
    /*
        pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
            stream.write_all(&[0])?;
            stream.write_all(&[0])?;
            stream.write_all(&[0])?;
            stream.write_all(self.len)?;
            stream.write_all(self.id)?;

            Ok(())
        }
    */
}
