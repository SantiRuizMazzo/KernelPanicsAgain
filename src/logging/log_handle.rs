use std::sync::mpsc::Sender;

pub enum LogMessage {
    Log(String),
    Kill,
}

#[derive(Clone)]
pub struct LogHandle {
    sender: Sender<LogMessage>,
}

impl LogHandle {
    pub fn new(sender: Sender<LogMessage>) -> Self {
        Self { sender }
    }

    pub fn log(&self, message: &str) -> Result<(), String> {
        self.sender
            .send(LogMessage::Log(message.to_string()))
            .map_err(|e| e.to_string())
    }

    pub fn kill(&self) -> Result<(), String> {
        self.sender
            .send(LogMessage::Kill)
            .map_err(|e| e.to_string())
    }
}
