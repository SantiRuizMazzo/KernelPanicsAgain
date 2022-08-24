use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Write},
    net::{IpAddr, Ipv4Addr, TcpListener, TcpStream},
    str::FromStr,
    thread,
    time::Duration,
};

use super::{
    thread_pool::{
        pool_creation_error::PoolCreationError, web_server_thread_pool::WebServerThreadPool,
    },
    tracker_info::{
        peer_tracker_info::{PeerTrackerInfo, PeerTrackerState},
        torrent_registry::TorrentRegistry,
    },
};
pub struct WebServer {
    pool: WebServerThreadPool,
}

impl WebServer {
    pub fn new() -> Result<WebServer, PoolCreationError> {
        let torrent_registry = TorrentRegistry::new();
        let pool = WebServerThreadPool::new(5, torrent_registry)?;
        Ok(WebServer { pool })
    }
    pub fn run(&self) -> Result<(), String> {
        self.pool.execute(WebServer::track_stats);
        let listener = TcpListener::bind("localhost:7878").map_err(|err| err.to_string())?;
        for stream in listener.incoming().flatten() {
            self.pool
                .execute(|registry| WebServer::handle_connection(stream, registry));
            println!("Connection established!");
        }
        Ok(())
    }

    fn track_stats(torrent_registry: TorrentRegistry) {
        loop {
            let _ = torrent_registry.save_to_json();
            thread::sleep(Duration::from_secs(60));
        }
    }

    fn handle_connection(mut stream: TcpStream, torrent_registry: TorrentRegistry) {
        let request: Vec<String> = BufReader::new(&mut stream)
            .lines()
            .filter_map(|line| line.ok())
            .take_while(|line| !line.is_empty())
            .collect();

        if request.len() < 3 {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n");
            return;
        }

        let split: Vec<String> = request[0].split(' ').map(|part| part.to_string()).collect();
        if split.len() < 3 {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n");
            return;
        }

        let peer_ip = match stream.peer_addr() {
            Ok(address) => address.ip(),
            Err(_) => {
                let _ = stream
                    .write_all(b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n");
                return;
            }
        };

        let response = if split[0] == "GET" {
            process_get_request(&split[1], torrent_registry, peer_ip)
        } else {
            b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n".to_vec()
        };

        let _ = stream.write_all(&response);
    }
}

fn process_get_request(
    requested_uri: &str,
    torrent_registry: TorrentRegistry,
    peer_ip: IpAddr,
) -> Vec<u8> {
    if let Some(query_params) = requested_uri.strip_prefix("/announce?") {
        let query = query_params.to_string();
        process_announce(query, torrent_registry, peer_ip)
    } else if requested_uri.starts_with("/stats?get_data") {
        get_stats_data()
    } else if requested_uri.starts_with("/stats") {
        get_stats_html()
    } else {
        b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n".to_vec()
    }
}

fn process_announce(query: String, torrent_registry: TorrentRegistry, peer_ip: IpAddr) -> Vec<u8> {
    match params_from_query(query) {
        Ok(params) => match process_params(params, torrent_registry, peer_ip) {
            Ok(response) => response,
            Err(response) => response,
        },
        Err(_) => b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n".to_vec(),
    }
}

fn process_params(
    params: HashMap<String, String>,
    torrent_registry: TorrentRegistry,
    peer_ip: IpAddr,
) -> Result<Vec<u8>, Vec<u8>> {
    let err_response = b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n".to_vec();

    let info_hash = params
        .get("info_hash")
        .ok_or_else(|| err_response.clone())?;
    let peer_id = params.get("peer_id").ok_or_else(|| err_response.clone())?;
    let port = params.get("port").ok_or_else(|| err_response.clone())?;
    let port: u16 = port
        .parse()
        .map_err(|_| b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n".to_vec())?;

    if let Some(compact) = params.get("compact") {
        if compact != "1" {
            return Err(err_response);
        }
    }

    let ip = match params.get("ip") {
        Some(ip) => Ipv4Addr::from_str(ip).map_err(|_| {
            b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n".to_vec()
        })?,
        None => match peer_ip {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => return Err(err_response),
        },
    };

    let mut numwant: usize = 50;
    if let Some(value) = params.get("numwant") {
        let temp = value.parse().map_err(|_| {
            b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n".to_vec()
        })?;
        if temp <= 50 {
            numwant = temp
        }
    }

    let response = get_tracker_response(torrent_registry.clone(), info_hash.clone(), numwant);

    if let Some(event) = params.get("event") {
        let event = &event.to_lowercase()[..];
        let state = match event {
            "started" => PeerTrackerState::Started,
            "completed" => PeerTrackerState::Completed,
            "stopped" => PeerTrackerState::Stopped,
            _ => return Err(err_response),
        };

        let peer = PeerTrackerInfo::new(peer_id.to_string(), ip, port, state);
        torrent_registry.insert(info_hash.to_string(), peer);
    }
    Ok(response)
}

fn get_tracker_response(
    torrent_registry: TorrentRegistry,
    info_hash: String,
    numwant: usize,
) -> Vec<u8> {
    match torrent_registry.get_bencoded_contents(info_hash, numwant) {
        Ok(mut content) => {
            let length = content.len();
            let mut response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n"
            )
            .as_bytes()
            .to_vec();
            response.append(&mut content);
            response
        }
        Err(_) => b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n".to_vec(),
    }
}

fn params_from_query(query: String) -> Result<HashMap<String, String>, String> {
    let params: Vec<String> = query.split('&').map(|param| param.to_string()).collect();
    let mut param_dict: HashMap<String, String> = HashMap::new();

    for param in params {
        let key_value_vec: Vec<String> = param.split('=').map(|s| s.to_string()).collect();
        let key = key_value_vec.get(0).ok_or("Error getting key")?.to_string();
        let value = key_value_vec
            .get(1)
            .ok_or("Error getting value")?
            .to_string();
        param_dict.insert(key, value);
    }
    Ok(param_dict)
}

fn get_stats_data() -> Vec<u8> {
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("./tracker_server/server/data.json");
    match contents {
        Ok(json) => {
            let length = json.len();
            format!(
                "{status_line}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {length}\r\n\r\n{json}"
            ).as_bytes().to_vec()
        }
        _ => {
            let json = "{}".to_string();
            let length = json.len();
            format!(
                "{status_line}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {length}\r\n\r\n{json}"
            ).as_bytes().to_vec()
        }
    }
}

fn get_stats_html() -> Vec<u8> {
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("./tracker_server/server/stats.html");
    match contents {
        Ok(html) => {
            let length = html.len();
            format!(
                "{status_line}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {length}\r\n\r\n{html}"
            ).as_bytes().to_vec()
        }
        _ => "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n"
            .to_string()
            .as_bytes()
            .to_vec(),
    }
}
