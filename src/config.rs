pub struct Config {
    pub tcp_port: i32,
}

impl Config {
    pub fn new(tcp_port: i32) -> Config {
        Config { tcp_port }
    }

    pub fn get_server_address(&self) -> String {
        "0.0.0.0:".to_owned() + &(self.tcp_port).to_string()
    }

    pub fn get_client_address(&self) -> String {
        "localhost:".to_owned() + &(self.tcp_port).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correctly_tcp_port() {
        let config = Config::new(8081);
        assert_eq!(8081, config.tcp_port);
    }

    #[test]
    fn get_the_correct_tcp_port_server_address() {
        let config = Config::new(8081);
        let address = "0.0.0.0:".to_owned() + &(8081).to_string();
        assert_eq!(address, config.get_server_address());
    }

    #[test]
    fn get_the_correct_tcp_port_client_address() {
        let config = Config::new(8081);
        let address = "localhost:".to_owned() + &(8081).to_string();
        assert_eq!(address, config.get_client_address());
    }
}
