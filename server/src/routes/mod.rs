//! This module contains all the routes for chouette.

pub mod login;
pub mod new_user;
pub mod imap_account;

use std::io::Cursor;
use rocket::response::Response;

#[get("/")]
/// The index route of the server.
pub fn index<'a>() -> Response<'a> {
    Response::build()
        .sized_body(Cursor::new(include_str!("../../dist/index.html")))
        .finalize()
}

#[get("/main.js")]
/// The route to the elm script.
pub fn script<'a>() -> Response<'a> {
    Response::build()
        .sized_body(Cursor::new(include_str!("../../dist/main.js")))
        .finalize()
}

