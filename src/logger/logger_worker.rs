use chrono::prelude::*;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{self};
use std::thread;

use super::torrent_logger::Message;

pub struct LoggerWorker {
    pub thread: Option<thread::JoinHandle<()>>,
}

impl LoggerWorker {
    pub fn new(receiver: mpsc::Receiver<Message>, mut file: File) -> LoggerWorker {
        let thread = thread::spawn(move || loop {
            if let Ok(message) = receiver.recv() {
                match message {
                    Message::Log(string) => {
                        println!("{}", string);
                        let local: DateTime<Local> = Local::now();
                        if let Err(e) = writeln!(file, "{} - {}", local, string) {
                            eprintln!("Couldn't write to file: {}", e);
                        }
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            };

            //println!("Worker {} got a job; executing.", id);
        });
        LoggerWorker {
            thread: Some(thread),
        }
    }
}
