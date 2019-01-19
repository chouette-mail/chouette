//! This module contains the routes of the API.

use std::collections::HashMap;

use diesel::prelude::*;

use futures_state_stream::StateStream;
use tokio::prelude::*;
use tokio::prelude::stream::futures_ordered;
use tokio_imap::proto::ResponseData;
use tokio_imap::client::{ImapClient, TlsClient};
use tokio_imap::types::{Response, Status};
use imap_proto::builders::command::CommandBuilder;

use warp::Filter;
use warp::filters::BoxedFilter;
use warp::reply::Reply;
use warp::http::response::Builder;
use warp::http::header::SET_COOKIE;
use warp::cookie::cookie;

use crate::SERVER_CONFIG;
use crate::mailbox::Mailbox;
use crate::utils::{error_400, error_500, ok_response};
use crate::auth::user::User;
use crate::auth::session::Session;
use crate::auth::imap_account::ImapAccount;
use crate::routes::session;


/// Creates the route to register new users.
pub fn register() -> BoxedFilter<(impl Reply, )> {

    warp::post2()
        .and(warp::path("api"))
        .and(warp::path("new-user"))
        .and(warp::body::form())
        .map(|argument: HashMap<String, String>| {

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

            let connection = connect!(SERVER_CONFIG);

            match user.save(&connection) {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to save user to the database: {:?}", e);
                    return error_500("");
                },
            }

            info!("Saved {:?}", user);
            ok_response("")
        })
        .boxed()

}

/// Creates the route to log users, and save their sessions.
pub fn login() -> BoxedFilter<(impl Reply, )> {

    warp::post2()
        .and(warp::path("api"))
        .and(warp::path("login"))
        .and(warp::body::form())
        .map(|arguments: HashMap<String, String>| {

            info!("Login requested");

            let username = extract_or_bad_request!(arguments, "username");
            let password = extract_or_bad_request!(arguments, "password");

            let connection = connect!(SERVER_CONFIG);

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

            Builder::new()
                .header(SET_COOKIE, format!("EXAUTH={}; SameSite=Strict; HttpOpnly", session.secret))
                .body("")
                .unwrap()
        })
        .boxed()
}

/// Creates the route to add new imap accounts to users.
pub fn add_imap_account() -> BoxedFilter<(impl Reply, )> {

    warp::post2()
        .and(cookie("EXAUTH"))
        .and_then(session)
        .and(warp::path("api"))
        .and(warp::path("add-imap-account"))
        .and(warp::body::form())
        .map(move |session: Session, arguments: HashMap<String, String>| {

            info!("New imap account requested");

            let imap_server = extract_or_bad_request!(arguments, "server");
            let username = extract_or_bad_request!(arguments, "username");
            let password = extract_or_bad_request!(arguments, "password");

            let connection = connect!(SERVER_CONFIG);

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
        })
        .boxed()
}

/// Creates the route that allows user to test an IMAP account.
///
/// It will return an error 400 if the connection didn't succeed, and an OK 200 if everything went
/// right.
pub fn test_imap_account() -> BoxedFilter<(impl Reply, )> {

    warp::post2()
        .and(cookie("EXAUTH"))
        .and_then(session)
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
                Some(Response::Done { status: Status::Ok, .. }) => ok_response(""),
                _ => error_400("")
            }
        })
        .boxed()
}

/// Creates the route that allows the users to fetch the different mailboxes they own.
pub fn fetch_mailboxes() -> BoxedFilter<(impl Reply, )> {

    warp::post2()
        .and(cookie("EXAUTH"))
        .and_then(session)
        .and(warp::path("api"))
        .and(warp::path("get-mailboxes"))
        .and(warp::body::form())
        .and_then(move |session: Session, _arguments: HashMap<String, String>| {

            info!("Mailboxes requested");

            let connection = match SERVER_CONFIG.database.connect() {
                Ok(c) => c,
                Err(_) => panic!(),
            };

            let query_results = {
                use crate::schema::imap_accounts::dsl::*;
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
                Ok(r) => ok_response(r),
                Err(_) => error_500(String::from("")),
            }
        })
        .boxed()
}
