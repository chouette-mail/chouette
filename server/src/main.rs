#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::io::Cursor;
use rocket::response::Response;
use rocket::request::Form;
use rocket::http::{Cookie, Cookies};
use clap::App;

use chouette::auth::user::User;
use chouette::auth::session::Session;
use chouette::auth::remote_account::ImapAccount;

#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(FromForm)]
struct NewUserForm {
    username: String,
    email: String,
    password: String,
}

#[derive(FromForm)]
struct ImapAccountForm {
    server: String,
    username: String,
    password: String,
}

#[get("/")]
fn index<'a>() -> Response<'a> {
    Response::build()
        .sized_body(Cursor::new(include_str!("../dist/index.html")))
        .finalize()
}

#[get("/main.js")]
fn script<'a>() -> Response<'a> {
    Response::build()
        .sized_body(Cursor::new(include_str!("../dist/main.js")))
        .finalize()
}

#[post("/login", data = "<login>")]
fn login<'a>(mut cookies: Cookies, login: Form<LoginForm>) -> Result<Response<'a>, chouette::Error> {

    let db = chouette::SERVER_CONFIG.database.connect()?;
    let user = User::authenticate(&login.username, &login.password, &db)?;
    let session = user.save_session(&db)?;

    cookies.add_private(Cookie::new("EXAUTH", session.secret));

    Ok(Response::build()
        .sized_body(Cursor::new(include_str!("../dist/main.js")))
        .finalize())

}

#[post("/new-user", data = "<user>")]
fn new_user<'a>(user: Form<NewUserForm>) -> Result<Response<'a>, chouette::Error> {

    let user = User::create(&user.username, &user.email, &user.password)?;
    let db = chouette::SERVER_CONFIG.database.connect()?;
    user.save(&db)?;

    Ok(Response::build()
        .sized_body(Cursor::new(""))
        .finalize())
}

#[post("/test-imap-account", data = "<account>")]
fn test_imap_account<'a>(mut cookies: Cookies, account: Form<ImapAccountForm>) -> Result<Response<'a>, chouette::Error> {
    let session = cookies
        .get_private("EXAUTH")
        .ok_or(chouette::Error::SessionDoesNotExist)?;

    let db = chouette::SERVER_CONFIG.database.connect()?;
    Session::from_secret(session.value(), &db)?;
    ImapAccount::test(&account.server, &account.username, &account.password)?;

    Ok(Response::build()
        .sized_body(Cursor::new(""))
        .finalize())
}

#[post("/add-imap-account", data = "<account>")]
fn add_imap_account<'a>(mut cookies: Cookies, account: Form<ImapAccountForm>) -> Result<Response<'a>, chouette::Error> {
    let session = cookies
        .get_private("EXAUTH")
        .ok_or(chouette::Error::SessionDoesNotExist)?;

    let db = chouette::SERVER_CONFIG.database.connect()?;
    let session = Session::from_secret(session.value(), &db)?;

    ImapAccount::new(session.user_id, &account.server, &account.username, &account.password)
        .save(&db)?;

    Ok(Response::build()
        .sized_body(Cursor::new(""))
        .finalize())
}

#[post("/get-mailboxes")]
fn get_mailboxes<'a>(mut cookies: Cookies) -> Result<Response<'a>, chouette::Error> {

    let session = cookies
        .get_private("EXAUTH")
        .ok_or(chouette::Error::SessionDoesNotExist)?;

    let db = chouette::SERVER_CONFIG.database.connect()?;
    let session = Session::from_secret(session.value(), &db)?;
    let imap_accounts = ImapAccount::from_user_id(session.user_id, &db)?;

    let mut mailboxes = vec![];
    for account in imap_accounts {
        mailboxes.push(account.fetch_mailboxes()?);
    }

    Ok(Response::build()
        .sized_body(Cursor::new(serde_json::to_string(&mailboxes)?))
        .finalize())

}

fn main() {

    let _ = App::new("Chouette Mail")
        .version("0.0.0")
        .author("Thomas Forgione <thomas@forgione.fr>")
        .about("A cool webmail written in Rust and Elm")
        .get_matches();

    rocket::ignite()
        .mount("/", routes![index, script])
        .mount("/api", routes![
            login,
            new_user,
            test_imap_account,
            add_imap_account,
            get_mailboxes,
        ])
        .launch();

}
