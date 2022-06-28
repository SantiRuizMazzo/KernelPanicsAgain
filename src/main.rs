use patk_bittorrent_client::torrent_client::run;

fn main() {
    match run() {
        Ok(()) => println!("Successful download 😎🤙"),
        Err(error) => println!("{error}"),
    }
}
