use std::{
    env,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
};

use chrono::Local;

macro_rules! err {
    ($($arg:tt)*) => {
        eprintln!("\x09\x1b[31m{}\x1b[0m", format!($($arg)*))
    };
}

fn main() {
    let listener = match TcpListener::bind("0.0.0.0:8080") {
        Ok(listener) => listener,
        Err(e) => {
            //panic!("Failed to bind: {}", e)
            err!("Failed to bind: {}", e);
            return;
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => err!("Connection failed: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    // Handle the connection
    print!(
        "{} ({}) ",
        stream.peer_addr().unwrap().ip(),
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let req_buffer = BufReader::new(&stream);
    let req: Vec<_> = req_buffer
        .lines()
        .map(|line| match line {
            Ok(line) => {
                // println!("line read: {}", line);
                line
            }
            Err(e) => {
                err!("Error reading line: {}", e);
                String::new()
            }
        })
        .take_while(|line| !line.is_empty())
        .collect();

    let mut file = String::new();

    if req[0].starts_with("GET") {
        if req[0].contains("/ ") {
            file = String::from("index.html");
            println!("GET /");
        } else {
            file = req[0].split_whitespace().nth(1).unwrap().to_string();
            file.remove(0);
            println!("GET /{}", file);
        }
    }

    let args: Vec<String> = env::args().collect();

    let mut path = String::new();

    if args.len() == 2 && args[1].ends_with("/") {
        path.push_str(&args[1]);
    } else {
        path.push_str("app/");
    }
    path.push_str(&file);

    let (contents, status_line) = match std::fs::read_to_string(path) {
        Ok(contents) => (contents, "HTTP/1.1 200 OK"),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                err!("404 Error: file not found");
                (
                    String::from("<h1>404 Error: file not found</h1>"),
                    "HTTP/1.1 404 Not Found",
                )
            } else {
                err!("Error reading file: {}", e);

                (
                    String::from("<h1>Internal Server Error</h1>"),
                    "HTTP 500 Internal Server Error",
                )
            }
        }
    };

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
