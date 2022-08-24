use super::{log_handle::LogHandle, log_worker::LogWorker};
use std::{fs::OpenOptions, sync::mpsc};

pub struct Logger {
    handle: LogHandle,
    worker: LogWorker,
}

impl Logger {
    pub fn new(log_path: String) -> Result<Self, String> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .map_err(|e| e.to_string())?;
        let (log_tx, log_rx) = mpsc::channel();

        Ok(Self {
            handle: LogHandle::new(log_tx),
            worker: LogWorker::new(log_rx, file),
        })
    }

    pub fn handle(&self) -> LogHandle {
        self.handle.clone()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.handle.kill();
        let _ = self.worker.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::thread;

    #[test]
    fn test_logging_in_different_threads() -> Result<(), String> {
        let logger = Logger::new(Config::new()?.log_path())?;
        let handle = logger.handle();
        handle.log("Este es un test de log")?;
        handle.log("Este es otro de mis test de log")?;

        let thread = thread::spawn(move || {
            let thread_handle = logger.handle();
            thread_handle.log("Esto se mandó desde el thread")?;
            thread_handle.log("Esto también!!!")
        });
        let _ = thread.join();
        Ok(())
    }
}
