//! This module contains the routes that the server serves.

pub mod api;

use warp::Filter;
use warp::filters::BoxedFilter;
use warp::reply::Reply;

use crate::config::ServerConfig;
use crate::routes::api::{
    register,
    login,
    add_imap_account,
    test_imap_account,
    fetch_mailboxes
};

/// Creates a route to the index.html of the project.
pub fn index() -> BoxedFilter<(impl Reply, )> {
    warp::get2()
        .and(warp::path::end())
        .and(warp::fs::file("./dist/index.html"))
        .boxed()
}

/// Creates a route to the main.js of the project.
pub fn script() -> BoxedFilter<(impl Reply, )> {
    warp::path("main.js")
        .and(warp::path::end())
        .and(warp::fs::file("./dist/main.js"))
        .boxed()
}

/// Creates all the routes of chouette's server.
pub fn routes(config: ServerConfig) -> BoxedFilter<(impl Reply, )> {
    index()
        .or(script())
        .or(register(config.clone()))
        .or(login(config.clone()))
        .or(add_imap_account(config.clone()))
        .or(test_imap_account(config.clone()))
        .or(fetch_mailboxes(config.clone()))
        .boxed()
}
