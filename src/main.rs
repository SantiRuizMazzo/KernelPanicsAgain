use cli::torrent_client::run;

fn main() {
    match run() {
        Ok(()) => println!("Successful download 😎🤙"),
        Err(error) => println!("{error}"),
    }
}
