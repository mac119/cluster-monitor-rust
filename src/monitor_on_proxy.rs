use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn connect_to_redis(host: &str, password: &str) -> TcpStream {
    let mut stream = TcpStream::connect_timeout(
        &host.parse().expect("Invalid address"),
        Duration::from_secs(5),
    )
    .expect("Failed to connect to Redis");

    write!(stream, "AUTH {}\r\n", password).expect("Failed to send AUTH");

    // Read AUTH response
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));
    let mut line = String::new();
    reader.read_line(&mut line).expect("Failed to read AUTH response");

    stream
}

fn new_all_interface_conns() -> Vec<TcpStream> {
    let file = match File::open("proxyInfo") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Vec::new();
        }
    };

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // First line is the password
    let password = match lines.next() {
        Some(Ok(pwd)) => pwd,
        _ => {
            eprintln!("Error: failed to read password from proxyInfo");
            return Vec::new();
        }
    };

    let mut conns = Vec::new();

    for line in lines {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 2 {
            eprintln!("Warning: invalid line in proxyInfo: {}", line);
            continue;
        }

        let addr = format!("{}:{}", fields[0], fields[1]);
        let conn = connect_to_redis(&addr, &password);
        conns.push(conn);
    }

    conns
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
    let conns = new_all_interface_conns();

    let mut handles = Vec::new();

    for stream in conns {
        let handle = thread::spawn(move || {
            monitor(stream);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
