#[macro_use]
extern crate log;

use std::net::{SocketAddrV4, Ipv4Addr};
use std::collections::HashMap;
use clap::{Arg, App};
use warp::Filter;

use chouette::config::ServerConfig;
use chouette::auth::User;

macro_rules! extract_or_empty {
    ($map: expr, $param: expr) => {
        match $map.get($param) {
            Some(o) => o,
            None => {
                error!("Missing parameter {} in request", $param);
                return warp::reply();
            },
        }
    }
}

macro_rules! connect {
    ($config: expr) => {
        match $config.database.connect() {
            Ok(c) => c,
            Err(e) => {
                error!("Couldn't connect to the database: {:?}", e);
                return warp::reply();
            },
        }
    }
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
        .get_matches();

    stderrlog::new()
        .modules(vec![module_path!(), "chouette"])
        .verbosity(1 + matches.occurrences_of("verbose") as usize)
        .init()
        .expect("Couldn't initialize logger");

    info!("Parsing config...");
    let config = ServerConfig::from("config.toml")
        .expect("Couldn't parse config file");
    info!("Done!");

    info!("Connecting to the database...");
    info!("Done!");

    info!("Building routes...");
    let index = warp::get2()
        .and(warp::path::end())
        .and(warp::fs::file("./dist/index.html"));

    let js = warp::path("main.js")
        .and(warp::path::end())
        .and(warp::fs::file("./dist/main.js"));

    let config_clone = config.clone();
    let register = warp::post2()
        .and(warp::path("api"))
        .and(warp::path("new-user"))
        .and(warp::body::form())
        .map(move |argument: HashMap<String, String>| {
            let username = extract_or_empty!(argument, "username");
            let email = extract_or_empty!(argument, "email");
            let password = extract_or_empty!(argument, "password");

            let user = match User::create(username, email, password) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to create user: {:?}", e);
                    return warp::reply()
                },
            };

            let connection = connect!(config_clone);

            match user.save(&connection) {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to save user to the database: {:?}", e);
                    return warp::reply();
                },
            }

            info!("Saved {:?}", user);
            warp::reply()
        });

    let config_clone = config.clone();
    let login = warp::post2()
        .and(warp::path("api"))
        .and(warp::path("login"))
        .and(warp::body::form())
        .map(move |argument: HashMap<String, String>| {

            let username = extract_or_empty!(argument, "username");
            let password = extract_or_empty!(argument, "password");

            let connection = connect!(config_clone);

            let user = match User::authenticate(username, password, &connection) {
                Some(user) => {
                    info!("User {} authenticated", username);
                    user
                },
                None => {
                    info!("User {} sent wrong password", username);
                    return warp::reply();
                },
            };

            let session = match user.save_session(&connection) {
                Ok(session) => {
                    info!("Session saved for user {}", username);
                    session
                },
                Err(e) => {
                    error!("Failed to save session for user {}: {:?}", username, e);
                    return warp::reply();
                },
            };

            warp::reply()
        });

    info!("Done!");

    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8000);

    info!("Server running on {}", socket.to_string());
    warp::serve(index.or(js).or(register).or(login))
        .run(socket);
}
