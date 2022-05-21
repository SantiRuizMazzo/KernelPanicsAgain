//use std::collections::VecDeque;
//use std::fs::File;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::mpsc::{self, Sender};

use super::logger_worker::LoggerWorker;

pub enum Message {
    Log(String),
    Terminate,
}
//use std::sync;
impl Logger {
    pub fn new(path: String) -> Result<Logger, String> {
        let (sender, receiver) = mpsc::channel();
        let path: &Path = Path::new(&path);
        let f: File = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
        {
            Ok(file) => file,
            Err(error) => {
                return Err(error.to_string());
            }
        };
        let worker = LoggerWorker::new(receiver, f);
        Ok(Logger { sender, worker })
    }
}
impl Drop for Logger {
    fn drop(&mut self) {
        // Either discard errors or panic for failed drop
        let _ = self.sender.send(Message::Terminate);
        if let Some(thread) = self.worker.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct Logger {
    pub sender: Sender<Message>,
    worker: LoggerWorker,
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    #[test]
    fn test_logging_in_different_threads() {
        let logger: Logger = Logger::new("tests/logtest.txt".to_string()).unwrap();
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(logger.sender.clone());
        let _ = thread::spawn(move || {
            let thread_sender = rx.recv().unwrap();
            let _ = thread_sender.send(Message::Log("Esto se mandó desde el thread".to_string()));
            let _ = thread_sender.send(Message::Log(
                "Esto también se mandó desde el thread".to_string(),
            ));
        })
        .join();
        let _ = logger
            .sender
            .send(Message::Log("Este es un test de log".to_string()));
        let _ = logger
            .sender
            .send(Message::Log("Este es otro de mis test de log".to_string()));
    }
}
