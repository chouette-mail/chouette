#[macro_use]
extern crate log;

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
                error!("Trying to get {}", $param);
                return warp::reply();
            },
        }
    }
}

fn main() {

    let matches = App::new("3D Model Converter")
        .version("0.1.0")
        .author("Thomas Forgione <thomas@forgione.fr>")
        .about("Converts 3D models")
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Display a more verbose output")
             .takes_value(false)
             .multiple(true))
        .get_matches();

    stderrlog::new()
        .modules(vec![module_path!(), "model_converter"])
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

    let register = warp::post2()
        .and(warp::path("new-user"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
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

            let connection = match config.database.connect() {
                Ok(c) => c,
                Err(e) => {
                    error!("Couldn't connect to the database: {:?}", e);
                    return warp::reply();
                },
            };

            match user.save(connection) {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to save user to the database: {:?}", e);
                    return warp::reply();
                },
            }

            info!("Saved {:?}", user);
            warp::reply()
        });

    info!("Done!");

    info!("Starting server");
    warp::serve(index.or(js).or(register))
        .run(([127, 0, 0, 1], 8000));
}
