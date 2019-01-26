//! This module contains the struct and functions for the login route.


use std::io::Cursor;

use rocket::response::Response;
use rocket::request::Form;
use rocket::http::{Cookie, Cookies};

use crate::{SERVER_CONFIG, Result};
use crate::auth::user::User;

#[derive(FromForm)]
/// A struct that serves for form veryfing.
pub struct LoginForm {
    /// The username in the form.
    username: String,

    /// The password in the form.
    password: String,
}

#[post("/login", data = "<login>")]
/// The login page.
pub fn login<'a>(mut cookies: Cookies, login: Form<LoginForm>) -> Result<Response<'a>> {

    let db = SERVER_CONFIG.database.connect()?;
    let user = User::authenticate(&login.username, &login.password, &db)?;
    let session = user.save_session(&db)?;

    cookies.add_private(Cookie::new("EXAUTH", session.secret));

    Ok(Response::build()
        .sized_body(Cursor::new(""))
        .finalize())
}
