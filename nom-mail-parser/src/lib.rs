//! This crate uses nom to parse mail content.

#![warn(missing_docs)]

#[macro_use]
extern crate nom;

use std::result;

pub mod parser;
pub use parser::parse;
pub use parser::parse_headers;

#[cfg(test)]
mod tests;

/// Public type of error of this crate.
pub type Error<'a> = nom::Err<&'a [u8]>;

/// Public type of result of this crate.
pub type Result<'a, T> = result::Result<T, Error<'a>>;

/// The different content types a mail can have.
#[derive(Debug)]
pub enum ContentType {
    /// A plain text mail.
    TextPlain,

    /// An HTML formatted mail.
    TextHtml,

    /// A multipart alternative, with sub mails.
    ///
    /// The Vec<u8> is the boundary to separate the mails.
    MultipartAlternative(Vec<u8>),
}

/// The content transfer encoding of a mail.
#[derive(Debug)]
pub enum ContentTransferEncoding {
    /// A quoted printable content.
    QuotedPrintable,

    /// Some base64-encoded content.
    Base64,
}

/// The different headers that appear in a mail.
#[derive(Debug)]
pub enum Header {
    /// The subject of the mail.
    Subject(String),

    /// The date of the mail.
    Date(String),

    /// The sender of the mail.
    From(String),

    /// The content type of the mail.
    ContentType(ContentType),

    /// The content transfer encoding of the mail.
    ContentTransferEncoding(ContentTransferEncoding),

    /// Some unknown header.
    Unknown(String),
}

/// The different types of body a mail can have.
#[derive(Debug)]
pub enum Body {
    /// It can be some raw content.
    Content(String),

    /// It can be a multipart mail, containing some sub mails.
    Multi(Vec<Mail>),
}

/// A collection of headers.
///
/// Provides some useful functions.
#[derive(Debug)]
pub struct Headers(pub Vec<Header>);

impl Headers {
    /// Returns the subject of the mail, if any.
    pub fn subject(&self) -> Option<&String> {
        for header in &self.0 {
            match header {
                Header::Subject(s) => return Some(s),
                _ => (),
            }
        }

        None
    }

    /// Looks for the boundary in the header of a mail.
    ///
    /// Returns none if it is not a multipart mail.
    fn boundary(&self) -> Option<&Vec<u8>> {
        for header in &self.0 {
            match header {
                Header::ContentType(ContentType::MultipartAlternative(b)) => return Some(b),
                _ => (),
            }
        }

        None
    }
}

/// The struct returned from our parse function.
#[derive(Debug)]
pub struct Mail {
    /// The headers of the mail.
    headers: Headers,

    /// The boy of the mail.
    body: Body,
}

impl Mail {
    /// Returns the subject of the mail, if any.
    pub fn subject(&self) -> Option<&String> {
        self.headers.subject()
    }
}

