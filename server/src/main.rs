#[macro_use]
extern crate log;

use std::net::{SocketAddrV4, Ipv4Addr};
use clap::{Arg, App};
use chouette::routes::routes;

fn parse_u16(content: &str) -> Result<u16, String> {
    content
        .parse::<u16>()
        .map_err(|e| format!("{:?}", e))
}

fn main() {

    let matches = App::new("Chouette Mail")
        .version("0.1.0")
        .author("Thomas Forgione <thomas@forgione.fr>")
        .about("A cool webmail written in Rust and Elm")
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Display a more verbose output")
             .takes_value(false)
             .multiple(true))
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .default_value("8000")
             .help("Port on which the server will listen")
             .validator(|p| parse_u16(&p).map(|_| ())))
        .get_matches();

    let verbosity = if cfg!(debug_assertions) {
        10
    } else {
        matches.occurrences_of("verbose")
    };

    let port = parse_u16(matches.value_of("port").unwrap()).unwrap();

    stderrlog::new()
        .modules(vec![module_path!(), "chouette"])
        .verbosity(verbosity as usize)
        .init()
        .expect("Couldn't initialize logger");

    let routes = routes();

    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    info!("Server running on {}", socket.to_string());
    warp::serve(routes).run(socket);
}
