#[macro_use]
extern crate log;

use std::net::{SocketAddrV4, Ipv4Addr};
use std::collections::HashMap;
use clap::{Arg, App};
use futures_state_stream::StateStream;
use tokio::prelude::*;
use tokio::prelude::stream::futures_ordered;
use tokio_imap::proto::ResponseData;
use tokio_imap::client::{ImapClient, TlsClient};
use tokio_imap::types::{self, AttrMacro};
use imap_proto::builders::command::{CommandBuilder, FetchBuilderMessages, FetchBuilderModifiers};
use diesel::prelude::*;
use warp::http::{header, StatusCode};
use warp::http::response::{self, Response};
use warp::Filter;
use warp::reject::Rejection;
use warp::cookie::cookie;

use chouette::config::ServerConfig;
use chouette::auth::{User, Session, ImapAccount};
use chouette::mailbox::Mailbox;

macro_rules! extract_or_panic {
    ($map: expr, $param: expr) => {
        match $map.get($param) {
            Some(o) => o,
            None => {
                error!("Missing parameter {} in request", $param);
                panic!();
            },
        }
    }
}
macro_rules! extract_or_bad_request {
    ($map: expr, $param: expr) => {
        match $map.get($param) {
            Some(o) => o,
            None => {
                error!("Missing parameter {} in request", $param);
                return error_400("");
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
                return error_500("");
            },
        }
    }
}

fn new_response<T>(code: StatusCode, body: T) -> Response<T> {
    response::Builder::new()
        .status(code)
        .body(body)
        .unwrap()
}

fn error_500<T>(body: T) -> Response<T> {
    return new_response(StatusCode::INTERNAL_SERVER_ERROR, body);
}

fn error_400<T>(body: T) -> Response<T> {
    return new_response(StatusCode::BAD_REQUEST, body);
}

fn ok_response<T>(body: T) -> Response<T> {
    response::Builder::new()
        .status(StatusCode::OK)
        .body(body)
        .unwrap()
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

            let username = extract_or_bad_request!(argument, "username");
            let email = extract_or_bad_request!(argument, "email");
            let password = extract_or_bad_request!(argument, "password");

            let user = match User::create(username, email, password) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to create user: {:?}", e);
                    return error_500("");
                },
            };

            let connection = connect!(config_clone);

            match user.save(&connection) {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to save user to the database: {:?}", e);
                    return error_500("");
                },
            }

            info!("Saved {:?}", user);
            ok_response("")
        });

    let config_clone = config.clone();
    let login = warp::post2()
        .and(warp::path("api"))
        .and(warp::path("login"))
        .and(warp::body::form())
        .map(move |arguments: HashMap<String, String>| {

            info!("Login requested");

            let username = extract_or_bad_request!(arguments, "username");
            let password = extract_or_bad_request!(arguments, "password");

            let connection = connect!(config_clone);

            let user = match User::authenticate(username, password, &connection) {
                Some(user) => {
                    info!("User {} authenticated", username);
                    user
                },
                None => {
                    info!("User {} sent wrong password", username);
                    return error_400("");
                },
            };

            let session = match user.save_session(&connection) {
                Ok(session) => {
                    info!("Session saved for user {}", username);
                    session
                },
                Err(e) => {
                    error!("Failed to save session for user {}: {:?}", username, e);
                    return error_500("");
                },
            };

            Response::builder()
                .header(header::SET_COOKIE, format!("EXAUTH={}; SameSite=Strict; HttpOpnly", session.secret))
                .body("")
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
        .and(warp::path("add-imap-account"))
        .and(warp::body::form())
        .map(move |session: Session, arguments: HashMap<String, String>| {

            info!("New imap account requested");

            let imap_server = extract_or_bad_request!(arguments, "server");
            let username = extract_or_bad_request!(arguments, "username");
            let password = extract_or_bad_request!(arguments, "password");

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

            ok_response("")
        });

    let config_clone = config.clone();
    let config_clone_bis = config.clone();
    let get_mailboxes = warp::post2()
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
        .and(warp::path("get-mailboxes"))
        .and(warp::body::form())
        .and_then(move |session: Session, arguments: HashMap<String, String>| {

            info!("Mailboxes requested");

            let connection = match config_clone_bis.database.connect() {
                Ok(c) => c,
                Err(_) => panic!(),
            };

            let query_results =
            {
                use chouette::schema::imap_accounts::dsl::*;
                imap_accounts
                    .filter(user_id.eq(session.user_id))
                    .select((id, user_id, server, username, password))
                    .get_results::<ImapAccount>(&connection)
            };

            let query_results = match query_results {
                Ok(a) => {
                    a
                },
                Err(e) => {
                    error!("Couldn't get imap accounts for user {}: {:?}", session.user_id, e);
                    panic!();
                },
            };

            futures_ordered(query_results.into_iter().map(|imap_account| {
                let server = imap_account.server.clone();
                let username = imap_account.username.clone();
                let username_bis = imap_account.username.clone();
                let username_ter = imap_account.username.clone();
                let password = imap_account.password.clone();

                info!("Connecting to the imap server {}...", server);

                TlsClient::connect(&server)
                    .expect("Yo")
                    .and_then(move |connection| {
                        info!("Connected to the imap server {}", server);

                        let command = CommandBuilder::login(&username, &password);
                        connection.1.call(command).collect()
                    })
                .and_then(move |(_, connection)| {
                    info!("Fetching the mailboxes for user {}", username_bis);
                    let command = CommandBuilder::list("", "*");
                    connection.call(command).collect()
                })
                .and_then(move |(response, _)| {
                    info!("Fetched the mailboxes for user {}", username_ter);

                    let mut mailboxes = vec![];
                    for data in response {
                        if let Ok(mailbox) = Mailbox::from_data(&data.parsed()) {
                            mailboxes.push(mailbox);
                        }
                    }

                    Ok(mailboxes)
                })
                .or_else(|_| future::err(warp::reject::not_found()))
            })).collect()
        })
        .map(|ref mailboxes: Vec<Vec<Mailbox>>| {
            match serde_json::to_string(mailboxes) {
                Ok(r) => {
                    println!("{}", r);
                    Response::new(r)
                },
                Err(_) => error_500(String::from("")),
            }
        });

    let config_clone = config.clone();
    let test_imap_account = warp::post2()
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
        .and(warp::path("test-imap-account"))
        .and(warp::body::form())
        .and_then(move |_session: Session, arguments: HashMap<String, String>| {

            info!("Test imap account requested");

            let server = extract_or_panic!(arguments, "server").clone();
            let username = extract_or_panic!(arguments, "username").clone();
            let password = extract_or_panic!(arguments, "password").clone();

            info!("Connecting to the imap server {}...", server);

            TlsClient::connect(&server)
                .expect("Yo")
                .and_then(move |connection| {
                    info!("Connected to the imap server {}", server);

                    let command = CommandBuilder::login(&username, &password);
                    connection.1.call(command).collect()
                })
            .or_else(|_| future::err(warp::reject::not_found()))
        })
        .map(|(response, _): (Vec<ResponseData>, TlsClient)| {
            match response.last().map(|x| x.parsed()) {
                Some(types::Response::Done { status: types::Status::Ok, .. }) => ok_response(""),
                _ => error_400("")
            }
        });

        info!("Done!");

    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7000);

    let routes = index
        .or(js)
        .or(register)
        .or(login)
        .or(new_imap_account)
        .or(test_imap_account)
        .or(get_mailboxes);

    info!("Server running on {}", socket.to_string());
    warp::serve(routes).run(socket);
}
