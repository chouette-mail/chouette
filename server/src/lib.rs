//! This crate contains everything needed by the server.

// Diesel generates massive amounts of warnings that are disabled with this.
#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

#![warn(missing_docs)]

macro_rules! impl_from_error {
    ($type: ty, $variant: path, $from: ty) => {
        impl From<$from> for $type {
            fn from(e: $from) -> $type {
                $variant(e)
            }
        }
    }
}

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;

#[macro_use]
pub mod utils;

#[macro_use]
extern crate tera;

pub mod config;
pub mod auth;
pub mod mailbox;
pub mod routes;

/// The diesel schema of the database.
#[allow(missing_docs)]
pub mod schema;

use config::ServerConfig;

/// The place where the config file is.
pub const CONFIG_FILE_LOCATION: &str = "config.toml";

lazy_static! {
    /// The configuration of the server.
    pub static ref SERVER_CONFIG: ServerConfig = ServerConfig::from(CONFIG_FILE_LOCATION)
        .expect("Couldn't parse config file");

    /// The templates our server will use.
    pub static ref TEMPLATES: tera::Tera = compile_templates!("assets/templates/*");
}

use std::{io, result};
use bcrypt::BcryptError;
use rocket::error::LaunchError;

/// The different errors that can occur when processing a request.
#[derive(Debug)]
pub enum Error {
    /// Couldn't connect to the database.
    DatabaseConnectionError(diesel::ConnectionError),

    /// Error while running a database request.
    DatabaseRequestError(diesel::result::Error),

    /// A session key was received but there was no such session.
    SessionDoesNotExist,

    /// A user try to log in but typed the wrong username or password.
    AuthenticationFailed,

    /// An argument is missing in a form.
    MissingArgumentInForm(String),

    /// An error occured while computing some bcrypt hash.
    BcryptError(BcryptError),

    /// An I/O error occured.
    IoError(io::Error),

    /// An error occured while establishing TLS connection.
    TlsError(native_tls::Error),

    /// An error occured during an IMAP communication.
    ImapError(imap::error::Error),

    // /// An error occured while parsing the content of an email.
    // ParseEmailError(nom_mail_parser::Error),

    /// An error occured during a serde operation.
    SerdeJsonError(serde_json::error::Error),

    /// An error occured while trying to create a mail.
    MailError(failure::Error),

    /// An error occured while trying to send a mail.
    SendMailError(lettre::smtp::error::Error),

    /// An error while building a mail.
    BuildMailError(lettre_email::error::Error),

    /// An error occured while rendering a template.
    TeraError(tera::Error),
}

impl_from_error!(Error, Error::DatabaseConnectionError, diesel::ConnectionError);
impl_from_error!(Error, Error::DatabaseRequestError, diesel::result::Error);
impl_from_error!(Error, Error::BcryptError, BcryptError);
impl_from_error!(Error, Error::IoError, io::Error);
impl_from_error!(Error, Error::ImapError, imap::error::Error);
impl_from_error!(Error, Error::TlsError, native_tls::Error);
impl_from_error!(Error, Error::SerdeJsonError, serde_json::error::Error);
// impl_from_error!(Error, Error::ParseEmailError, mailbox::mail::Error);
impl_from_error!(Error, Error::MailError, failure::Error);
impl_from_error!(Error, Error::SendMailError, lettre::smtp::error::Error);
impl_from_error!(Error, Error::BuildMailError, lettre_email::error::Error);
impl_from_error!(Error, Error::TeraError, tera::Error);

impl<T> From<(imap::error::Error, T)> for Error {
    fn from((e, _): (imap::error::Error, T)) -> Error {
        Error::ImapError(e)
    }
}

/// The result type of this library.
pub type Result<T> = result::Result<T, Error>;

/// Mounts all the routes and starts the server.
pub fn start() -> LaunchError {
    rocket::ignite()
        .mount("/", routes![routes::index, routes::script])
        .mount("/api", routes![
            routes::login::login,
            routes::new_user::new_user,
            routes::new_user::activate,
            routes::imap_account::test_imap_account,
            routes::imap_account::add_imap_account,
            routes::imap_account::fetch_mailboxes,
            routes::imap_account::fetch_subjects,
        ])
        .launch()
}
