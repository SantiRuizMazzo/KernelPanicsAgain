#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Piece {
    index: u32,
    pub begin: u32,
    id: u8,
    pub block: Vec<u8>,
}

impl Piece {
    pub fn new(index: u32, begin: u32, block: Vec<u8>) -> Piece {
        Piece {
            id: 1,
            index,
            begin,
            block,
        }
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
