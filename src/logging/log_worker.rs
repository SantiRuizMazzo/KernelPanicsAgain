use chrono::prelude::*;
use std::{
    fs::File,
    io::Write,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use super::log_handle::LogMessage;

pub struct LogWorker {
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl LogWorker {
    pub fn new(log_rx: Receiver<LogMessage>, mut file: File) -> Self {
        let thread = Some(thread::spawn(move || {
            while let LogMessage::Log(msg) = log_rx.recv().map_err(|e| e.to_string())? {
                let date_time = Local::now().naive_local().round_subsecs(2);
                writeln!(file, "{date_time} - {msg}").map_err(|e| e.to_string())?;
            }
            Ok(())
        }));

        Self { thread }
    }

    pub fn join(&mut self) -> Result<(), String> {
        self.thread
            .take()
            .ok_or("Error taking thread from log worker")?
            .join()
            .map_err(|_| "Error joining log worker")?
    }
}
