use std::{
    env,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
};

use log::{info, error,};
use env_logger;

use chrono::Local;


fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let path = env::args()
        .nth(1)
        .map(|path| {
            if !path.ends_with("/") {
                path + "/"
            } else {
                path
            }
        })
        .unwrap_or(String::from("app/"));

    let listener = match TcpListener::bind("0.0.0.0:8080") {
        Ok(listener) => {
            info!("Bound to localhost:8080");
            listener
        } ,
        Err(e) => {
            error!("Failed to bind: {}", e);
            return;
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, path.as_str()),
            Err(e) => error!("Connection failed: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream, path: &str) {
    // Handle the connection

    let req_buffer = BufReader::new(&stream);
    let req: Vec<_> = req_buffer
        .lines()
        .map(|line| match line {
            Ok(line) => line,
            Err(e) => {
                error!("Error reading line: {}", e);
                String::new()
            }
        })
        .take_while(|line| !line.is_empty())
        .collect();

    let mut file = String::new();

    if req[0].starts_with("GET") {
        if req[0].contains("/ ") {
            file = String::from("index.html");
            info!("GET / {}", stream.peer_addr().unwrap().ip());
        } else {
            file = req[0].split_whitespace().nth(1).unwrap().to_string();
            file.remove(0);
            info!("GET /{} {}", file, stream.peer_addr().unwrap().ip());
        }
    }

    file.insert_str(0, path);

    let (contents, status_line) = match std::fs::read_to_string(file) {
        Ok(contents) => (contents, "HTTP/1.1 200 OK"),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                error!("404 Error: file not found");
                (
                    String::from("<h1>404 Error: file not found</h1>"),
                    "HTTP/1.1 404 Not Found",
                )
            }
            _ => {
                error!("Error reading file: {}", e);
                (
                    String::from("<h1>Internal Server Error</h1>"),
                    "HTTP 500 Internal Server Error",
                )
            }
        },
    };

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
