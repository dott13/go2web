use clap::{Arg, Command};
use scraper::{Html, Selector};
use std::{io::{Read, Write}, net::TcpStream};

fn main() {
    let matches = Command::new("go2web")
        .version("0.3")
        .author("dott13")
        .about("Make clean HTTP requests or search via CLI")
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
                .num_args(1..)
                .help("Search the term using a search engine"),
        )
        .get_matches();

        if let Some(url) = matches.get_one::<String>("url") {
            handle_http_request(url);
        } else if let Some(terms) = matches.get_many::<String>("search") {
            let query = terms.map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
            perform_search(&query);
        } else {
            println!("Use -h to see available options.");
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

        let parts: Vec<&str> = response.splitn(2, "\r\n\r\n").collect();

        if parts.len() < 2 {
            eprintln!("Malformed HTTP response (no header/body split)");
            return;
        }

        let body = parts[1];

        println!("----- CLEAN TEXT OUTPUT -----");

        let document = Html::parse_document(body);
        let selector = Selector::parse("body").unwrap();

        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            println!("{}", text.trim());
        }
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

    fn perform_search(term: &str) {
        let query = urlencoding::encode(term);
        let host = "html.duckduckgo.com";
        let path = format!("/html/?q={}", query);

        let addr = format!("{}:80", host);

        let mut stream = TcpStream::connect(&addr).expect("Failed to connect to search engine");

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );

        stream
            .write_all(request.as_bytes())
            .expect("Failed to send search request");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("Failed to read search request");

        let parts: Vec<&str> = response.splitn(2, "\r\n\r\n").collect();
        if parts.len() < 2 {
            eprintln!("Malformed HTTP response from DuckDuckGo.");
            return;
        }

        let body = parts[1];
        let document = Html::parse_document(body);

        let results_selector = Selector::parse("a.result__a").unwrap();

        println!("----- TOP 10 SEARCH RESULTS -----");
        for (i, element) in document.select(&results_selector).take(10).enumerate() {
            let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            let link = element.value().attr("href").unwrap_or("N/A");
            println!("{}. {}\n   {}", i + 1, title, link);
        }
    }
}
