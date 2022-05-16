use patk_bittorrent_client::torrent_client::run;

fn main() {
    if let Ok(()) = run() {
        println!("Successful run");
    }
}
