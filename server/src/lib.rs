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

pub mod config;
pub mod auth;
pub mod mailbox;

/// The diesel schema of the database.
#[allow(missing_docs)]
pub mod schema;

