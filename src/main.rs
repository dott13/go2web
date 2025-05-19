use clap::{Arg, Command};
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use urlencoding::decode;
use std::{fs, io::{self, Read, Write}, net::TcpStream, path::Path};

fn ensure_cache_dir() {
    let _ = fs::create_dir_all(".cache");
}

fn main() {
    let matches = Command::new("go2web")
        .version("0.9")
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
        .arg(
            Arg::new("accept")
                .long("accept")
                .num_args(1)
                .value_parser(["html", "json"])
                .default_value("html")
                .help("Specify accepted content type (html or json)"),
        )
        .get_matches();

        let accept_type = matches
             .get_one::<String>("accept")
            .unwrap()
            .as_str();

        if let Some(url) = matches.get_one::<String>("url") {
            handle_http_request(url, accept_type);
        } else if let Some(terms) = matches.get_many::<String>("search") {
            let query = terms.map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
            perform_search(&query);
        } else {
            println!("Use -h to see available options.");
        }

    fn handle_http_request(url: &str, accept: &str) {
        ensure_cache_dir();

        let hash = Sha256::digest(url.as_bytes());
        let hash_hex = format!("{:x}", hash);
        let cache_path = format!(".cache/{}.html", hash_hex);
        
        if Path::new(&cache_path).exists() {
            println!("Cached response found.");
            print!("Use cached response? (y/N): ");
            io::stdout().flush().unwrap();
    
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
    
            if input == "y" || input == "yes" {
                let cached = fs::read_to_string(&cache_path).expect("Failed to read cache");
                if accept == "json" {
                    display_json(&cached);
                } else {
                    display_html(&cached);
                }
                return;
            } else {
                println!("Fetching fresh copy...");
            }
        }

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
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: go2web/1.0\r\nAccept: {}\r\nConnection: close\r\n\r\n",
            path, host,
            if accept == "json" { "application/json" } else { "text/html" }
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

        let header = parts[0];
        let body = parts[1];

        if header.starts_with("HTTP/1.1 3") {
            if let Some(location_line) = header.lines().find(|line| line.to_lowercase().starts_with("location")) {
                let location = location_line.splitn(2, ":").nth(1).unwrap_or("").trim();
                println!("Redirecting to: {}", location);
                if location.starts_with("http://") {
                    handle_http_request(location, accept);
                    return;
                } else {
                    println!("Cannot follow non-http redirects: {}", location);
                    return;
                }
            }
        }

        fs::write(&cache_path, body).expect("Failed to write to cache");
        println!("response has been added to cache");
        if accept == "json" || header.contains("application/json") {
            display_json(body);
        } else {
            display_html(body);
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

    fn display_html(body: &str) {
        let document = Html::parse_document(body);
        let selector = Selector::parse("body").unwrap();
    
        println!("----- CLEAN TEXT OUTPUT -----");
        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            println!("{}", text.trim());
        }
    }

    fn display_json(body: &str) {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(json) => {
                println!("----- JSON RESPONSE -----");
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            }
            Err(err) => {
                eprintln!("❌ Failed to parse JSON: {}", err);
                println!("{}", body);
            }
        }
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

        let mut links = vec![];
        
        println!("----- TOP 10 SEARCH RESULTS -----");
        for (i, element) in document.select(&results_selector).take(10).enumerate() {
            let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            let href = element.value().attr("href").unwrap_or("N/A");
    
            let real_url = if href.contains("uddg=") {
                let encoded = href.split("uddg=").nth(1).unwrap_or("");
                decode(encoded).unwrap_or_else(|_| "N/A".into()).to_string()
            } else {
                href.to_string()
            };
    
            println!("{}. {}\n   {}", i + 1, title, real_url);
            links.push(real_url);
        }

        print!("\nSelect a result to open (1-{}), or 0 to skip: ", links.len());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if let Ok(index) = input.parse::<usize>() {
            if index > 0 && index <= links.len() {
                open_in_browser(&links[index - 1]);
            } else {
                println!("No link opened.");
            }
        } else {
            println!("Invalid input.");
        }
    }

    fn open_in_browser(url: &str) {
        #[cfg(target_os = "linux")]
        {
            if std::env::var("WSL_DISTRO_NAME").is_ok() {
                // Running inside WSL: use Windows start command
                let _ = std::process::Command::new("cmd.exe")
                    .args(["/C", "start", url])
                    .spawn();
            } else {
                // Regular Linux
                let _ = std::process::Command::new("xdg-open")
                    .arg(url)
                    .spawn();
            }
        }
    
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd.exe")
                .args(["/C", "start", url])
                .spawn();
        }
    
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(url)
                .spawn();
        }
    }
    
}
