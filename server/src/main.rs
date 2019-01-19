#[macro_use]
extern crate log;

use std::net::{SocketAddrV4, Ipv4Addr};
use clap::{Arg, App};
use chouette::routes::routes;

fn main() {

    #[allow(unused)]
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
        .get_matches();

    #[cfg(debug_assertions)]
    let verbosity = 10;
    #[cfg(not(debug_assertions))]
    let verbosity = matches.occurrences_of("verbose") as usize;

    stderrlog::new()
        .modules(vec![module_path!(), "chouette"])
        .verbosity(verbosity)
        .init()
        .expect("Couldn't initialize logger");

    info!("Connecting to the database...");
    info!("Done!");

    info!("Building routes...");
    let routes = routes();

    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7000);
    info!("Server running on {}", socket.to_string());
    warp::serve(routes).run(socket);
}
