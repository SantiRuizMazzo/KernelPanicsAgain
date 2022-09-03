use crate::server::tracker_info::torrent_registry::TorrentRegistry;

use super::pool_creation_error::PoolCreationError;
use super::web_server_worker::Message;
use super::web_server_worker::WebServerWorker;
use std::sync::{mpsc, Arc, Mutex};

pub struct WebServerThreadPool {
    workers: Vec<WebServerWorker>,
    sender: mpsc::Sender<Message>,
}

impl WebServerThreadPool {
    pub fn new(
        size: usize,
        torrent_registry: TorrentRegistry,
    ) -> Result<WebServerThreadPool, PoolCreationError> {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(WebServerWorker::new(
                id,
                Arc::clone(&receiver),
                torrent_registry.clone(),
            ));
        }

        Ok(WebServerThreadPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(TorrentRegistry) + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for WebServerThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
