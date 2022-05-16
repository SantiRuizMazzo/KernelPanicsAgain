use log::*;
//use std::collections::VecDeque;
//use std::fs::File;
use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
//use std::sync;

impl log::Log for Logger<'_> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
            if let Ok(mut f) = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.file_path)
            {
                let local: DateTime<Local> = Local::now(); // e.g. `2014-11-28T21:45:59.324310806+09:00`
                if let Err(e) = writeln!(f, "{} - {} - {}", local, record.level(), record.args()) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            };
        }
    }

    fn flush(&self) {}
}

#[derive(Debug, Copy, Clone)]
pub struct Logger<'a> {
    pub file_path: &'a Path,
}
