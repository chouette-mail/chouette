#[macro_use]
extern crate log;

use std::net::{SocketAddrV4, Ipv4Addr};
use std::collections::HashMap;
use clap::{Arg, App};
use warp::http::header;
use warp::http::response::Response;
use warp::Filter;
use warp::reject::Rejection;
use warp::cookie::cookie;

use chouette::config::ServerConfig;
use chouette::auth::{User, Session, ImapAccount};

macro_rules! extract_or_empty {
    ($map: expr, $param: expr) => {
        match $map.get($param) {
            Some(o) => o,
            None => {
                error!("Missing parameter {} in request", $param);
                return Response::new("");
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
                return Response::new("");
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

    #[cfg(debug_assertions)]
    let verbosity = 10;
    #[cfg(not(debug_assertions))]
    let verbosity = matches.occurrences_of("verbose") as usize;

    stderrlog::new()
        .modules(vec![module_path!(), "chouette"])
        .verbosity(verbosity)
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

            info!("New user requested");

            let username = extract_or_empty!(argument, "username");
            let email = extract_or_empty!(argument, "email");
            let password = extract_or_empty!(argument, "password");

            let user = match User::create(username, email, password) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to create user: {:?}", e);
                    return Response::new("")
                },
            };

            let connection = connect!(config_clone);

            match user.save(&connection) {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to save user to the database: {:?}", e);
                    return Response::new("");
                },
            }

            info!("Saved {:?}", user);
            Response::new("")
        });

    let config_clone = config.clone();
    let login = warp::post2()
        .and(warp::path("api"))
        .and(warp::path("login"))
        .and(warp::body::form())
        .map(move |arguments: HashMap<String, String>| {

            info!("Login requested");

            let username = extract_or_empty!(arguments, "username");
            let password = extract_or_empty!(arguments, "password");

            let connection = connect!(config_clone);

            let user = match User::authenticate(username, password, &connection) {
                Some(user) => {
                    info!("User {} authenticated", username);
                    user
                },
                None => {
                    info!("User {} sent wrong password", username);
                    return Response::new("");
                },
            };

            let session = match user.save_session(&connection) {
                Ok(session) => {
                    info!("Session saved for user {}", username);
                    session
                },
                Err(e) => {
                    error!("Failed to save session for user {}: {:?}", username, e);
                    return Response::new("");
                },
            };

            Response::builder()
                .header(header::SET_COOKIE, format!("EXAUTH={}; SameSite=Strict; HttpOpnly", session.secret))
                .body("")
                .ok()
                .unwrap()
        });

    let config_clone = config.clone();
    let config_clone_bis = config.clone();
    let new_imap_account = warp::post2()
        .and(cookie("EXAUTH"))
        .and_then(move |key: String| -> Result<Session, Rejection> {
            let connection = match config_clone.database.connect() {
                Ok(c) => c,
                Err(e) => {
                    error!("Couldn't connect to the database: {:?}", e);
                    panic!()
                },
            };

            let session = match Session::from_secret(&key, &connection) {
                Some(s) => {
                    info!("Found session for user {}", s.user_id);
                    s
                },
                None => {
                    info!("No session found");
                    panic!()
                },
            };

            Ok(session)
        })
        .and(warp::path("api"))
        .and(warp::path("new-imap-account"))
        .and(warp::body::form())
        .map(move |session: Session, arguments: HashMap<String, String>| {

            info!("New imap account requested");

            let imap_server = extract_or_empty!(arguments, "imap");
            let username = extract_or_empty!(arguments, "username");
            let password = extract_or_empty!(arguments, "password");

            let connection = connect!(config_clone_bis);

            let imap_account = ImapAccount::new(session.user_id, imap_server, username, password);

            match imap_account.save(&connection) {
                Ok(_) => {
                    info!("Imap account saved succesfully for user {}", session.user_id);
                },
                Err(e) => {
                    error!("Couldn't connect save imap account to the database: {:?}", e);
                    panic!()
                },
            }

            Response::new("")
        });

    info!("Done!");

    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7000);

    let routes = index
        .or(js)
        .or(register)
        .or(login)
        .or(new_imap_account);

    info!("Server running on {}", socket.to_string());
    warp::serve(routes).run(socket);
}
