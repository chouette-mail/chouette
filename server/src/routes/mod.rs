//! This module contains all the routes for chouette.

pub mod login;
pub mod new_user;
pub mod imap_account;

use std::fs::File;
use rocket::response::Response;
use crate::Result;

#[get("/")]
/// The index route of the server.
pub fn index<'a>() -> Result<Response<'a>> {
    let file = File::open("dist/index.html")?;
    Ok(Response::build()
        .sized_body(file)
        .finalize())
}

#[get("/main.js")]
/// The route to the elm script.
pub fn script<'a>() -> Result<Response<'a>> {
    let file = File::open("dist/main.js")?;
    Ok(Response::build()
        .sized_body(file)
        .finalize())
}

