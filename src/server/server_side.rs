use crate::config::Config;
use crate::messages::message_parser;
use std::io::Read;
use std::net::{TcpListener, TcpStream};

pub struct ServerSide {
    config: Config,
    peer_id: String,
}

impl ServerSide {
    pub fn new(port: i32) -> ServerSide {
        ServerSide {
            config: Config::new(port),
            peer_id: "empty_peer_id".to_string(),
        }
    }

    pub fn init_server(&mut self) {
        if self.run_server().is_ok() {
        } else {
            println!("Server connection fail\n");
        }
    }

    fn run_server(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.config.get_server_address())?;
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut bytes = [0; 69];
                    if stream.read_exact(&mut bytes).is_ok() {
                        println!("Handshake receive: {:?}", bytes);
                        if message_parser::is_handshake_message(bytes) {
                            self.send_handshake_response(stream, bytes);
                        }
                    } else {
                        println!("Reading fail")
                    }
                }
                Err(e) => {
                    println!("Connection fail {:?}", e);
                }
            }
        }

        drop(listener);
        Ok(())
    }

    pub fn set_peer_id(&mut self, peer_id: String) {
        self.peer_id = peer_id;
    }

    fn send_handshake_response(&self, mut stream: TcpStream, bytes: [u8; 69]) {
        let mut new_handshake = message_parser::parse_handshake(bytes);
        new_handshake.set_peer_id(self.peer_id.clone());
        if new_handshake.send(&mut stream).is_ok() {
        } else {
            println!("Handshake response fail")
        }
    }
}
