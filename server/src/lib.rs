//! This crate contains everything needed by the server.

// Diesel generates massive amounts of warnings that are disabled with this.
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
pub mod utils;

pub mod config;
pub mod auth;
pub mod mailbox;

/// The diesel schema of the database.
#[allow(missing_docs)]
pub mod schema;

use config::ServerConfig;

lazy_static! {
    pub static ref SERVER_CONFIG: ServerConfig = ServerConfig::from("config.toml")
        .expect("Couldn't parse config file");
}

use std::{io, result};
use bcrypt::BcryptError;

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

    /// An error occured during a serde operation.
    SerdeJsonError(serde_json::error::Error),
}

impl_from_error!(Error, Error::DatabaseConnectionError, diesel::ConnectionError);
impl_from_error!(Error, Error::DatabaseRequestError, diesel::result::Error);
impl_from_error!(Error, Error::BcryptError, BcryptError);
impl_from_error!(Error, Error::IoError, io::Error);
impl_from_error!(Error, Error::ImapError, imap::error::Error);
impl_from_error!(Error, Error::TlsError, native_tls::Error);
impl_from_error!(Error, Error::SerdeJsonError, serde_json::error::Error);

impl<T> From<(imap::error::Error, T)> for Error {
    fn from((e, _): (imap::error::Error, T)) -> Error {
        Error::ImapError(e)
    }
}

/// The result type of this library.
pub type Result<T> = result::Result<T, Error>;
