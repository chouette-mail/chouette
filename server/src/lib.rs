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
