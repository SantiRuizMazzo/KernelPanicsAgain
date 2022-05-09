use crate::client::client_side::ClientSide;
pub fn run() {
    let client = ClientSide::new();
    println!("{}", client.peer_id);
}
