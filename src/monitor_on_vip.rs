use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use clap::Parser;

/// Redis cluster monitor tool - VIP mode
/// Discovers all proxy instances behind a VIP and monitors them concurrently.
#[derive(Parser, Debug)]
#[command(name = "monitor-on-vip")]
struct Args {
    /// Redis server host (VIP address)
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Redis server port
    #[arg(short, long, default_value = "6379")]
    port: String,

    /// Redis server password
    #[arg(short = 'a', long)]
    passwd: String,
}

struct RedisConfig {
    host: String,
    port: String,
    passwd: String,
}

struct InterfaceConn {
    stream: TcpStream,
    proxy_id: String,
}

fn get_new_conn(config: &RedisConfig) -> InterfaceConn {
    let addr = format!("{}:{}", config.host, config.port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().expect("Invalid address"),
        Duration::from_secs(5),
    )
    .expect("Failed to connect to Redis");

    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));

    // AUTH
    write!(stream, "AUTH {}\r\n", config.passwd).expect("Failed to send AUTH");
    let mut line = String::new();
    reader.read_line(&mut line).expect("Failed to read AUTH response");

    // proxyId
    write!(stream, "proxyId\r\n").expect("Failed to send proxyId");
    line.clear();
    reader.read_line(&mut line).expect("Failed to read proxyId bulk length");
    line.clear();
    reader.read_line(&mut line).expect("Failed to read proxyId value");

    let proxy_id = line.trim_end_matches("\r\n").trim_end_matches('\n').to_string();

    InterfaceConn { stream, proxy_id }
}

fn find_all_interface(config: &RedisConfig) -> HashMap<String, TcpStream> {
    let mut conn_map: HashMap<String, TcpStream> = HashMap::new();

    for _ in 0..300 {
        let if_conn = get_new_conn(config);
        if conn_map.contains_key(&if_conn.proxy_id) {
            // Duplicate proxy, close the connection
            drop(if_conn.stream);
        } else {
            println!("Discovered proxy: {}", if_conn.proxy_id);
            conn_map.insert(if_conn.proxy_id, if_conn.stream);
        }
    }

    println!("Total proxies discovered: {}", conn_map.len());
    conn_map
}

fn monitor(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));

    write!(stream, "monitor\r\n").expect("Failed to send MONITOR");

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                eprintln!("Connection closed");
                return;
            }
            Ok(_) => {
                if line == "+OK\r\n" {
                    continue;
                }

                if line.starts_with("-ERR ") {
                    eprintln!(
                        "Redis return error message: {}",
                        line[1..].trim_end_matches("\r\n")
                    );
                    return;
                }

                // Strip the leading '+' or '$' character from Redis inline reply
                if !line.is_empty() {
                    print!("{}", &line[1..]);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.passwd.is_empty() {
        eprintln!("Error: --passwd is required");
        std::process::exit(1);
    }

    let config = RedisConfig {
        host: args.host,
        port: args.port,
        passwd: args.passwd,
    };

    let conn_map = find_all_interface(&config);

    let mut handles = Vec::new();

    for (_proxy_id, stream) in conn_map {
        let handle = thread::spawn(move || {
            monitor(stream);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
