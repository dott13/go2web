use clap::{Arg, Command};

fn main() {
    let matches = Command::new("go2web")
        .version("0.1")
        .author("dott13")
        .about("Make raw HTTP requests or search via CLI")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .takes_value(true)
                .help("Make an HTTP GET request to a URL"),
        )
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .takes_value(true)
                .help("Search the term using a search engine"),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(clap::ArgAction::Help),
        )
        .get_matches();

    if let Some(url) = matches.get_one::<String>("url") {
        println!("You selected URL: {}", url);
    } else if let Some(term) = matches.get_one::<String>("search") {
        println!("You selected search: {}", term);
    }
}
