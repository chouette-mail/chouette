//! This module contains the routes of the API.

use std::collections::HashMap;

use diesel::prelude::*;

use futures_state_stream::StateStream;
use tokio::prelude::*;
use tokio::prelude::stream::futures_ordered;
use tokio_imap::proto::ResponseData;
use tokio_imap::client::{ImapClient, ImapConnectFuture, TlsClient};
use tokio_imap::types::{Response as ImapResponse, Status};
use imap_proto::builders::command::{Command, CommandBuilder};

use warp::Filter;
use warp::filters::BoxedFilter;
use warp::reply::Reply;
use warp::http::response::{Response as HttpResponse, Builder};
use warp::http::header::SET_COOKIE;
use warp::cookie::cookie;
use warp::reject::Rejection;

use crate::{Error, SERVER_CONFIG};
use crate::mailbox::Mailbox;
use crate::utils::{error_400, error_500, ok_response};
use crate::auth::user::User;
use crate::auth::session::Session;
use crate::auth::remote_account::ImapAccount;
use crate::routes::session;


/// Creates the route to register new users.
pub fn register() -> BoxedFilter<(impl Reply, )> {

    warp::post2()
        .and(warp::path("api"))
        .and(warp::path("new-user"))
        .and(warp::body::form())
        .and_then(|argument: HashMap<String, String>| -> Result<HttpResponse<&str>, Rejection> {

            info!("New user requested");

            let username = extract!(argument, "username")?;
            let email = extract!(argument, "email")?;
            let password = extract!(argument, "password")?;

            let user = User::create(username, email, password)?;
            let connection = SERVER_CONFIG.database.connect()?;

            user.save(&connection)?;
            info!("Saved {:?}", user);

            Ok(ok_response(""))
        })
        .boxed()

}

/// Creates the route to log users, and save their sessions.
pub fn login() -> BoxedFilter<(impl Reply, )> {

    warp::post2()
        .and(warp::path("api"))
        .and(warp::path("login"))
        .and(warp::body::form())
        .and_then(|arguments: HashMap<String, String>| -> Result<HttpResponse<&str>, Rejection> {

            info!("Login requested");

            let username = extract!(arguments, "username")?;
            let password = extract!(arguments, "password")?;
            let connection = SERVER_CONFIG.database.connect()?;
            let user = User::authenticate(username, password, &connection)?;
            let session = user.save_session(&connection)?;

            Ok(Builder::new()
                .header(SET_COOKIE, format!("EXAUTH={}; SameSite=Strict; HttpOpnly", session.secret))
                .body("")
                .unwrap())
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
        .and_then(move |session: Session, arguments: HashMap<String, String>| -> Result<HttpResponse<&str>, Rejection> {

            info!("New imap account requested");

            let imap_server = extract!(arguments, "server")?;
            let username = extract!(arguments, "username")?;
            let password = extract!(arguments, "password")?;
            let connection = SERVER_CONFIG.database.connect()?;
            let imap_account = ImapAccount::new(session.user_id, imap_server, username, password);

            imap_account.save(&connection)?;
            Ok(ok_response(""))
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
        .and_then(|_session: Session, arguments: HashMap<String, String>| -> Result<(ImapConnectFuture, Command), Rejection> {

            info!("Test imap account requested");

            let server = extract!(arguments, "server")?;
            let username = extract!(arguments, "username")?;
            let password = extract!(arguments, "password")?;
            let command = CommandBuilder::login(username, password);

            info!("Connecting to the imap server {}...", server);

            Ok((TlsClient::connect(&server).map_err(|e| Into::<Error>::into(e))?, command))
        })
        .and_then(|(connection, command): (ImapConnectFuture, Command)| {
            let future = connection.map_err(|e| {
                let e = Into::<Error>::into(e);
                Into::<Rejection>::into(e)
            });

            (future, Ok(command))
        })
        .and_then(|((_, connection), command): ((ResponseData, TlsClient), Command) | {
            info!("Connected to imap server, logging in...");
            connection.call(command).collect().map_err(|e| {
                let e = Into::<Error>::into(e);
                Into::<Rejection>::into(e)
            })
        })
        .map(|(response, _): (Vec<ResponseData>, TlsClient)| {
            match response.last().map(|x| x.parsed()) {
                Some(ImapResponse::Done { status: Status::Ok, .. }) => {
                    info!("Logged in succesfully");
                    ok_response("")
                },
                _ => {
                    error!("Log in failed");
                    error_400("")
                },
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
        .and_then(move |session: Session, _arguments: HashMap<String, String>| -> Result<Vec<ImapAccount>, Rejection> {

            info!("Mailboxes requested");

            let connection = SERVER_CONFIG.database.connect()?;

            let query_results = {
                use crate::schema::imap_accounts::dsl::*;
                imap_accounts
                    .filter(user_id.eq(session.user_id))
                    .select((id, user_id, server, username, password))
                    .get_results::<ImapAccount>(&connection)
                    .map_err(Into::<Error>::into)?
            };

            Ok(query_results)
        })
        .and_then(|query_results: Vec<ImapAccount> | {

            futures_ordered(query_results.into_iter().map(|imap_account| {
                let server = imap_account.server.clone();
                let username = imap_account.username.clone();
                let username_bis = imap_account.username.clone();
                let username_ter = imap_account.username.clone();
                let password = imap_account.password.clone();

                info!("Connecting to the imap server {}...", server);

                TlsClient::connect(&server)
                    .into_future()
                    .and_then(|x| x)
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
