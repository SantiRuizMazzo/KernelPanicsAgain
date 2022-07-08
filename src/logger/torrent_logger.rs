use std::fs::{File, OpenOptions};
use std::sync::mpsc::{self, Sender};

use super::logger_worker::LoggerWorker;
use crate::config::Config;

pub enum LogMessage {
    Log(String),
    Kill,
}

pub struct Logger {
    sender: Sender<LogMessage>,
    worker: LoggerWorker,
}

impl Logger {
    pub fn new(config: Config) -> Result<Logger, String> {
        let (sender, receiver) = mpsc::channel();
        let f: File = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&config.get_log_path())
        {
            Ok(file) => file,
            Err(error) => {
                return Err(error.to_string());
            }
        };
        let worker = LoggerWorker::new(receiver, f);
        Ok(Logger { sender, worker })
    }

    pub fn get_sender(&self) -> Sender<LogMessage> {
        self.sender.clone()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.sender.send(LogMessage::Kill);
        if let Some(thread) = self.worker.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    #[test]
    fn test_logging_in_different_threads() -> Result<(), String> {
        let logger: Logger = Logger::new(Config::new()?)?;
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(logger.sender.clone());
        let _ = thread::spawn(move || {
            let thread_sender = rx.recv().unwrap();
            let _ =
                thread_sender.send(LogMessage::Log("Esto se mandó desde el thread".to_string()));
            let _ = thread_sender.send(LogMessage::Log(
                "Esto también se mandó desde el thread".to_string(),
            ));
        })
        .join();
        let _ = logger
            .sender
            .send(LogMessage::Log("Este es un test de log".to_string()));
        let _ = logger.sender.send(LogMessage::Log(
            "Este es otro de mis test de log".to_string(),
        ));
        Ok(())
    }
}
