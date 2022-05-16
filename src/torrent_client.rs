use crate::client::client_side::ClientSide;
use crate::logger::torrent_logger::*;
use log::*;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{boxed::*, path::Path};
pub fn run() -> Result<(), io::Error> {
    let path: &Path = Path::new("logtest.txt");
    let data = Arc::new(Mutex::new(Logger { file_path: path }));
    let mut children = vec![];
    for _ in 0..10 {
        let data = Arc::clone(&data);
        children.push(thread::spawn(move || {
            if let Ok(data) = data.lock() {
                let _res = log::set_boxed_logger(Box::new(*data))
                    .map(|()| log::set_max_level(LevelFilter::Info));
                let client = ClientSide::new();
                info!("Peer ID: {}", client.peer_id);
            }
        }));
    }
    for child in children {
        // Esperar que terminen los threads
        let _ = child.join();
    }

    Ok(())
}
