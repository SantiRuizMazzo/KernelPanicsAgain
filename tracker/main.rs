use server::web_server::WebServer;

mod server;

fn main() -> Result<(), String> {
    let server = WebServer::new().map_err(|err| err.to_string())?;
    server.run()
}
