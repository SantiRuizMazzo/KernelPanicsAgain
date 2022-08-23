use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

#[derive(Clone)]
pub struct Config {
    tcp_port: usize,
    log_path: String,
    download_path: String,
    torrent_time_slice: usize,
    max_download_connections: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tcp_port: 8081,
            log_path: "log.txt".to_string(),
            download_path: "downloads".to_string(),
            torrent_time_slice: 10,
            max_download_connections: 20,
        }
    }
}

impl Config {
    pub fn new() -> Result<Self, String> {
        let file = File::open("config.txt").map_err(|e| e.to_string())?;
        let mut config = Self::default();

        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| e.to_string())?;
            let value = Self::value_from_line(&line);

            if line.starts_with("tcp_port") {
                config.tcp_port = usize::from_str(&value).map_err(|e| e.to_string())?;
            } else if line.starts_with("log_path") {
                config.log_path = value;
            } else if line.starts_with("download_path") {
                config.download_path = value;
            } else if line.starts_with("torrent_time_slice") {
                config.torrent_time_slice = usize::from_str(&value).map_err(|e| e.to_string())?;
            } else if line.starts_with("max_download_connections") {
                config.max_download_connections =
                    usize::from_str(&value).map_err(|e| e.to_string())?;
            }
        }

        Ok(config)
    }

    pub fn download_path(&self) -> String {
        self.download_path.clone()
    }

    pub fn log_path(&self) -> String {
        self.log_path.clone()
    }

    pub fn get_tcp_port(&self) -> usize {
        self.tcp_port
    }

    pub fn get_max_download_connections(&self) -> usize {
        self.max_download_connections
    }

    pub fn torrent_time_slice(&self) -> usize {
        self.torrent_time_slice
    }

    fn value_from_line(line: &str) -> String {
        let split_line: Vec<&str> = line.rsplit('=').collect();
        split_line[0].to_string()
    }

    pub fn server_address(&self) -> String {
        format!("localhost:{}", self.tcp_port)
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
