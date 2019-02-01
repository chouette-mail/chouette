//! This module contains the routes related to the imap accounts.


use std::io::Cursor;
use rocket::response::Response;
use rocket::request::Form;
use rocket::http::Cookies;

use crate::{SERVER_CONFIG, Error, Result};
use crate::auth::session::Session;
use crate::auth::remote_account::ImapAccount;

#[derive(FromForm)]
/// A struct that serves the purpose of verifying the form.
pub struct ImapAccountForm {
    /// The url of the server.
    server: String,

    /// The username to log in the IMAP account.
    username: String,

    /// The password to log in the IMAP account.
    password: String,
}

#[post("/test-imap-account", data = "<account>")]
/// Route that tests a IMAP account can succesfully log.
pub fn test_imap_account<'a>(mut cookies: Cookies, account: Form<ImapAccountForm>) -> Result<Response<'a>> {
    let session = cookies
        .get_private("EXAUTH")
        .ok_or(Error::SessionDoesNotExist)?;

    let db = SERVER_CONFIG.database.connect()?;
    Session::from_secret(session.value(), &db)?;
    ImapAccount::test(&account.server, &account.username, &account.password)?;

    Ok(Response::build()
        .sized_body(Cursor::new(""))
        .finalize())
}

#[post("/add-imap-account", data = "<account>")]
/// Route that adds an IMAP account to a user.
pub fn add_imap_account<'a>(mut cookies: Cookies, account: Form<ImapAccountForm>) -> Result<Response<'a>> {
    let session = cookies
        .get_private("EXAUTH")
        .ok_or(Error::SessionDoesNotExist)?;

    let db = SERVER_CONFIG.database.connect()?;
    let session = Session::from_secret(session.value(), &db)?;

    ImapAccount::create(session.user_id, &account.server, &account.username, &account.password)
        .save(&db)?;

    Ok(Response::build()
        .sized_body(Cursor::new(""))
        .finalize())
}

#[post("/get-mailboxes")]
/// A route that fetches all the mailboxes for the IMAP accounts of a user.
pub fn fetch_mailboxes<'a>(mut cookies: Cookies) -> Result<Response<'a>> {

    let session = cookies
        .get_private("EXAUTH")
        .ok_or(Error::SessionDoesNotExist)?;

    let db = SERVER_CONFIG.database.connect()?;
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

#[derive(FromForm)]
/// A struct that serves the purpose of verifying the fetch subjects route.
pub struct FetchSubjectsForm {
    /// The name of the mailbox to fetch
    inbox: String,
}

#[post("/get-subjects", data = "<form>")]
/// A route that feches all the subjects of a mailbox of an IMAP account.
pub fn fetch_subjects<'a>(mut cookies: Cookies, form: Form<FetchSubjectsForm>) -> Result<Response<'a>> {
    let session = cookies
        .get_private("EXAUTH")
        .ok_or(Error::SessionDoesNotExist)?;

    let db = SERVER_CONFIG.database.connect()?;
    let session = Session::from_secret(session.value(), &db)?;
    let imap_accounts = ImapAccount::from_user_id(session.user_id, &db)?;

    let imap_account = match imap_accounts.first() {
        Some(x) => (x),
        None => {
            return Ok(Response::build()
                .sized_body(Cursor::new(""))
                .finalize())
        },
    };

    let subjects = imap_account.fetch_subjects(&form.inbox, 1, 20)?;

    Ok(Response::build()
        .sized_body(Cursor::new(serde_json::to_string(&subjects)?))
        .finalize())
}
