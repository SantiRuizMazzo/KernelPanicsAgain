#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NotInterested {
    len: u32,
    id: u8,
}

impl NotInterested {
    pub fn new() -> NotInterested {
        NotInterested { len: 1, id: 3 }
    }
}
