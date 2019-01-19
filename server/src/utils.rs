//! This module contains some useful functions and macros.

use warp::http::StatusCode;
use warp::http::response::{Builder, Response};

/// Returns a HTTP response with the given status code and body.
pub fn new_response<T>(code: StatusCode, body: T) -> Response<T> {
    Builder::new()
        .status(code)
        .body(body)
        .unwrap()
}

/// Returns an error 500 response with the given body.
pub fn error_500<T>(body: T) -> Response<T> {
    new_response(StatusCode::INTERNAL_SERVER_ERROR, body)
}

/// Returns an error 400 with the given body.
pub fn error_400<T>(body: T) -> Response<T> {
    new_response(StatusCode::BAD_REQUEST, body)
}

/// Returns an OK 200 response with the given body.
pub fn ok_response<T>(body: T) -> Response<T> {
    new_response(StatusCode::OK, body)
}

/// Extracts the value of the option and panic if none.
///
/// This should not be used, except for debugging. It produces slightly better output than unwrap.
#[macro_export]
macro_rules! extract_or_panic {
    ($map: expr, $param: expr) => {
        match $map.get($param) {
            Some(o) => o,
            None => {
                error!("Missing parameter {} in request", $param);
                panic!();
            },
        }
    }
}

/// Extracts the value of the option and return bad request if none.
#[macro_export]
macro_rules! extract_or_bad_request {
    ($map: expr, $param: expr) => {
        match $map.get($param) {
            Some(o) => o,
            None => {
                error!("Missing parameter {} in request", $param);
                return crate::utils::error_400("");
            },
        }
    }
}

/// Tries to connect to the database and return an error 500 if it's not possible.
#[macro_export]
macro_rules! connect {
    ($config: expr) => {
        match $config.database.connect() {
            Ok(c) => c,
            Err(e) => {
                error!("Couldn't connect to the database: {:?}", e);
                return crate::utils::error_500("");
            },
        }
    }
}
