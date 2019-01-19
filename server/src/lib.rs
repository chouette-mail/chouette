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
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod utils;

pub mod config;
pub mod auth;
pub mod mailbox;
pub mod routes;

/// The diesel schema of the database.
#[allow(missing_docs)]
pub mod schema;

use config::ServerConfig;

lazy_static! {
    static ref SERVER_CONFIG: ServerConfig = ServerConfig::from("config.toml")
        .expect("Couldn't parse config file");
}

use std::{io, fmt, error, result};
use bcrypt::BcryptError;
use warp::http::StatusCode;
use warp::reject::Rejection;

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
}

impl Error {

    /// Returns the status code corresponding to the error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::DatabaseRequestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DatabaseConnectionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BcryptError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SessionDoesNotExist => StatusCode::BAD_REQUEST,
            Error::AuthenticationFailed => StatusCode::BAD_REQUEST,
            Error::MissingArgumentInForm(_) => StatusCode::BAD_REQUEST,
            Error::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl_from_error!(Error, Error::DatabaseConnectionError, diesel::ConnectionError);
impl_from_error!(Error, Error::DatabaseRequestError, diesel::result::Error);
impl_from_error!(Error, Error::BcryptError, BcryptError);
impl_from_error!(Error, Error::IoError, io::Error);

impl fmt::Display for Error {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DatabaseRequestError(_) =>
                write!(w, "Error while executing a request to the database"),

            Error::DatabaseConnectionError(_) =>
                write!(w, "Couldn't connect to the database"),

            Error::BcryptError(_) =>
                write!(w, "Error while computing a hash"),

            Error::SessionDoesNotExist =>
                write!(w, "Session does exist"),

            Error::AuthenticationFailed =>
                write!(w, "Authentication failed"),

            Error::MissingArgumentInForm(a) =>
                write!(w, "The argument \"{}\" is missing in a form", a),

            Error::IoError(_) =>
                write!(w, "An I/O occured"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::DatabaseRequestError(e) => Some(e),
            Error::DatabaseConnectionError(e) => Some(e),
            Error::BcryptError(e) => Some(e),
            Error::SessionDoesNotExist => None,
            Error::AuthenticationFailed => None,
            Error::MissingArgumentInForm(_) => None,
            Error::IoError(e) => Some(e),
        }
    }
}

impl From<Error> for Rejection {
    fn from(error: Error) -> Rejection {
        warp::reject::custom(error)
    }
}

/// The result type of this library.
pub type Result<T> = result::Result<T, Error>;
