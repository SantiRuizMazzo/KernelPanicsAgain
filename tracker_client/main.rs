use std::{
    io::{Read, Write},
    net::TcpStream,
};

//const STATS: &[u8; 59] = b"GET /stats HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

fn main() {
    let mut stream = TcpStream::connect("localhost:7878").unwrap();
    let announce = b"GET /announce?info_hash=aaaaaaaaaaaaaaaaaaaa&peer_id=-UT3480-bbbbbbbbbbbb&port=7777&uploaded=0&downloaded=0&left=0&corrupt=0&key=10E0CE47&event=stopped&numwant=200&compact=1&no_peer_id=1&ip=192.168.43.188 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n".to_vec();
    stream.write_all(&announce).unwrap();
    let mut buf = Vec::<u8>::new();
    stream.read_to_end(&mut buf).unwrap();
    println!("{}", String::from_utf8_lossy(&buf))
}
