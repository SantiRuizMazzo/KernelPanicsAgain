use patk_bittorrent_client::torrent_client::run;

fn main() {
    match run() {
        Ok(()) => println!("Successful run"),
        Err(error) => println!("{error}"),
    }
}
