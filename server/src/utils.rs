//! This module contains some useful functions and macros.

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
macro_rules! extract {
    ($map: expr, $param: expr) => {
        $map.get($param).ok_or(crate::Error::MissingArgumentInForm(String::from($param)))
    }
}

