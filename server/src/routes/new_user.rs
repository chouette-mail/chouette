//! This module contains the route to register a new user.

use std::io::Cursor;

use rocket::response::Response;
use rocket::request::Form;

use crate::{SERVER_CONFIG, Result};
use crate::auth::user::User;

#[derive(FromForm)]
/// A struct that serves the purpose of veryifing the form.
pub struct NewUserForm {

    /// The username of the form.
    username: String,

    /// The email of the form.
    email: String,

    /// The password of the form.
    password: String,
}

#[post("/new-user", data = "<user>")]
/// The route to register new users.
pub fn new_user<'a>(user: Form<NewUserForm>) -> Result<Response<'a>> {

    let user = User::create(&user.username, &user.email, &user.password)?;
    let db = SERVER_CONFIG.database.connect()?;
    user.save(&db)?;

    Ok(Response::build()
        .sized_body(Cursor::new(""))
        .finalize())
}
