use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Clone)]
pub struct Config {
    tcp_port: u32,
    download_path: String,
    log_path: String,
}

impl Config {
    pub fn new() -> Result<Config, String> {
        let mut tcp_port = 8081;
        let mut download_path = "downloads".to_string();
        let mut log_path = "log.txt".to_string();

        if let Ok(file) = OpenOptions::new().read(true).open("config.txt") {
            for line in BufReader::new(file).lines() {
                let line = line.map_err(|err| err.to_string())?;
                let value = Config::get_value(line.clone());

                if line.contains("tcp_port") {
                    tcp_port = u32::from_str(&value).map_err(|err| err.to_string())?;
                } else if line.contains("download_path") {
                    download_path = value;
                } else if line.contains("log_path") {
                    log_path = value;
                }
            }
        };

        Ok(Config {
            tcp_port,
            download_path,
            log_path,
        })
    }

    pub fn get_download_path(&self) -> String {
        self.download_path.clone()
    }

    pub fn get_log_path(&self) -> String {
        self.log_path.clone()
    }

    fn get_value(line: String) -> String {
        let line: Vec<&str> = line.rsplit('=').collect();
        line[0].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correctly_configuration() -> Result<(), String> {
        let config = Config::new()?;
        assert_eq!(8081, config.tcp_port);
        assert_eq!("downloads/", config.download_path);
        assert_eq!("log.txt", config.log_path);
        Ok(())
    }
}
