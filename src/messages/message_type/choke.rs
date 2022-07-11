#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Choke {
    // 4 byte
    len: u32,
    // 1 byte
    id: u8,
}

impl Choke {
    pub fn new() -> Choke {
        Choke { len: 1, id: 0 }
    }
}
