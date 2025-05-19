use clap::{Arg, Command};
use std::{io::{Read, Write}, net::TcpStream};

fn main() {
    let matches = Command::new("go2web")
        .version("0.2")
        .author("dott13")
        .about("Make raw HTTP requests or search via CLI")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .num_args(1)
                .help("Make an HTTP GET request to a URL"),
        )
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .num_args(1)
                .help("Search the term using a search engine"),
        )
        .get_matches();

    if let Some(url) = matches.get_one::<String>("url") {
        handle_http_request(url);
    } else if let Some(term) = matches.get_one::<String>("search") {
        println!("You selected search: {}", term);
    }

    fn handle_http_request(url: &str) {
        let (host, path) = match parse_url(url) {
            Some(parts) => parts,
            None => {
                eprintln!("Invalid URL format. Use http://host/path");
                return;
            }
        };

        let addr = format!("{}:80", host);
        let mut stream = TcpStream::connect(&addr).expect("Failed to connect to server");

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );

        stream
            .write_all(request.as_bytes())
            .expect("Failed to send request");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect(" ----- BAD RESPONSE -----");

        println!(" ----- RAW RESPONSE -----");
        println!("{}", response);
    }

    fn parse_url(url: &str) -> Option<(String, String)>{
        if !url.starts_with("http://") {
            return None;
        }

        let without_scheme = &url[7..];
        let mut parts = without_scheme.splitn(2, '/');
        let host = parts.next()?.to_string();
        let path = format!("/{}", parts.next().unwrap_or(""));
        Some((host, path))
    }
}
