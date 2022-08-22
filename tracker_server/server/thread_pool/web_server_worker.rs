use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::server::tracker_info::torrent_registry::{self, TorrentRegistry};

type Job = Box<dyn FnOnce(TorrentRegistry) + Send + 'static>;

pub enum Message {
    NewJob(Job),
    Terminate,
}

pub struct WebServerWorker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl WebServerWorker {
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        torrent_registry: TorrentRegistry,
    ) -> WebServerWorker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    job(torrent_registry.clone());
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        WebServerWorker {
            id,
            thread: Some(thread),
        }
    }
}
