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

macro_rules! impl_from_error {
    ($type: ty, $variant: path, $from: ty) => {
        impl From<$from> for $type {
            fn from(e: $from) -> $type {
                $variant(e)
            }
        }
    }
}

/// Public type of error of this crate.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// An error while running nom parser.
    NomParseError,

    /// The content transfer encoding has an unknown value.
    UnknownContentTransferEncoding,

    /// An error while decoding a base64 encoded content.
    Base64DecodeError(base64::DecodeError),

    /// An errir while decoding an utf8 string.
    Utf8Error(std::str::Utf8Error),
}

impl<'a> From<nom::Err<&'a [u8]>> for Error {
    fn from(_: nom::Err<&'a [u8]>) -> Error {
        Error::NomParseError
    }
}

impl_from_error!(Error, Error::Base64DecodeError, base64::DecodeError);
impl_from_error!(Error, Error::Utf8Error, std::str::Utf8Error);


/// Public type of result of this crate.
pub type Result<T> = result::Result<T, Error>;

/// The different content types a mail can have.
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub enum ContentTransferEncoding {
    /// A quoted printable content.
    QuotedPrintable,

    /// Some base64-encoded content.
    Base64,
}

/// The different headers that appear in a mail.
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub enum Body {
    /// It can be some raw content.
    Content(String),

    /// It can be a multipart mail, containing some sub mails.
    Multi(Vec<Mail>),
}

/// A collection of headers.
///
/// Provides some useful functions.
#[derive(Debug, PartialEq, Eq)]
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

    /// Returns the from header of the mail, if any.
    pub fn from(&self) -> Option<&String> {
        for header in &self.0 {
            match header {
                Header::From(s) => return Some(s),
                _ => (),
            }
        }

        None
    }

    /// Returns the content type header of the mail, if any.
    pub fn content_type(&self) -> Option<&ContentType> {
        for header in &self.0 {
            match header {
                Header::ContentType(s) => return Some(s),
                _ => (),
            }
        }

        None
    }

    /// Returns the content transfer encoding header of the mail, if any.
    pub fn content_transfer_encoding(&self) -> Option<&ContentTransferEncoding> {
        for header in &self.0 {
            match header {
                Header::ContentTransferEncoding(s) => return Some(s),
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
#[derive(Debug, PartialEq, Eq)]
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

    /// Returns the from field of the mail, if any.
    pub fn from(&self) -> Option<&String> {
        self.headers.from()
    }

    /// Returns the content type field of the mail, if any.
    pub fn content_type(&self) -> Option<&ContentType> {
        self.headers.content_type()
    }

    /// Returns the plain body of the mail if any.
    pub fn plain_body(&self) -> Option<&String> {
        match &self.body {
            Body::Content(s) if self.content_type() == Some(&ContentType::TextPlain) => return Some(s),
            Body::Multi(mails) => {
                for mail in mails {
                    match (mail.content_type(), mail.plain_body()) {
                        (Some(&ContentType::TextPlain), Some(body)) => return Some(body),
                        _ => (),
                    }
                }
            },
            _ => return None,
        }

        None
    }

    /// Returns the html body of the mail if any.
    pub fn html_body(&self) -> Option<&String> {
        match &self.body {
            Body::Content(s) if self.content_type() == Some(&ContentType::TextHtml) => return Some(s),
            Body::Multi(mails) => {
                for mail in mails {
                    match (mail.content_type(), mail.html_body()) {
                        (Some(&ContentType::TextHtml), Some(body)) => return Some(body),
                        _ => (),
                    }
                }
            },
            _ => return None,
        }

        None
    }


}

