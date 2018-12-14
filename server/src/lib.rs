//! This crate contains everything needed by the server.

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

pub mod config;
pub mod auth;

/// The diesel schema of the database.
#[allow(missing_docs)]
pub mod schema;

