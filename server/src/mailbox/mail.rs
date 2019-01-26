//! This mail allows us to parse mails.

use std::result;
use std::collections::HashMap;
use std::io;
use std::str::Utf8Error;
use std::io::{Cursor, BufRead};
use std::string::FromUtf8Error;

use base64::DecodeError;

/// The error type of mail parsing.
#[derive(Debug)]
pub enum Error {
    /// An io error.
    IoError(io::Error),

    /// An utf-8 error occured while parsing the base 64 content of the mail.
    FromUtf8Error(FromUtf8Error),

    /// An utf-8 error occured while parsing the mail content.
    Utf8Error(Utf8Error),

    /// Unknown content type to parse.
    ///
    ///The paramter is the name of the unknown content type.
    UnknownContentType(String),

    /// Unknown encoding for content.
    UnknownEncoding(String),

    /// Error while decoding base 64 string.
    Base64Error(DecodeError),

    /// Expecting a boundary but received something else.
    ExpectingBoundary,

    /// Not a header while trying to parse a header.
    NotAHeader,

}

type Result<T> = result::Result<T, Error>;

impl_from_error!(Error, Error::IoError, io::Error);
impl_from_error!(Error, Error::FromUtf8Error, FromUtf8Error);
impl_from_error!(Error, Error::Utf8Error, Utf8Error);
impl_from_error!(Error, Error::Base64Error, DecodeError);

/// Decodes a UTF-8 encoded subject.
pub fn decode_subject(subject: &str) -> Result<String> {
    if subject.starts_with("=?") {
        let mut buf = String::new();
        let mut current = 0;
        let split = subject.split('?').collect::<Vec<_>>();

        loop {
            // The encoding is on current + 2
            if current + 3 >= split.len() {
                return Ok(buf);
            }

            if split[current + 2] == "B" {
                buf.push_str(&Encoding::Base64.decode(split[current + 3])?);
            }

            current += 4;
        }

    } else {
        Ok(subject.to_owned())
    }
}

/// The different encodings we support.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Encoding {
    /// The ascii encoding.
    QuotedPrintable,

    /// The base64 encoding
    Base64,
}

impl Encoding {
    /// Decodes the string passed as parameter depending on the encoding.
    pub fn decode(self, content: &str) -> Result<String> {
        match self {
            Encoding::QuotedPrintable => Ok(content.to_owned()),
            Encoding::Base64 => {
                Ok(String::from_utf8(base64::decode(content)?)?)
            },
        }
    }
}

/// The different content types we're able to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    /// Plain text mail.
    TextPlain,

    /// HTML text.
    TextHtml,

    /// A multipart alternative text.
    ///
    /// The string is the boundary. "--" is preprended to it.
    MultipartAlternative(String),
}

/// The possible header values.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum HeaderValue {
    /// A content type value.
    ContentType(ContentType),

    /// A content transfer encoding value.
    ContentTransferEncoding(Encoding),

    /// An unknown value represented as a string.
    Unknown(String),
}

impl HeaderValue {
    /// Creates a header value from a &str.
    pub fn from_str(key: &str, value: &str) -> Result<HeaderValue> {
        match key {
            "Content-Type" => {
                let split = value.split(';').map(str::trim).collect::<Vec<_>>();

                let content_type = match split[0] {
                    "multipart/alternative" | "multipart/mixed" => {

                        let mut boundary = None;
                        for subsplit_value in split[1..].iter() {
                            let subsplit = subsplit_value.split('"').map(str::trim).collect::<Vec<_>>();

                            if subsplit[0] != "boundary=" {
                                continue;
                            } else {
                                boundary = Some(ContentType::MultipartAlternative(format!("--{}", subsplit[1].trim())));
                            }
                        }

                        match boundary {
                            None => return Err(Error::ExpectingBoundary),
                            Some(b) => b,
                        }
                    },
                    "text/plain" => {
                        ContentType::TextPlain
                    },
                    "text/html" => {
                        ContentType::TextHtml
                    },
                    _ => return Err(Error::UnknownContentType(split[0].trim().to_owned())),
                };
                Ok(HeaderValue::ContentType(content_type))
            },

            "Content-Transfer-Encoding" => {
                let encoding = match value.trim() {
                    "base64" => Encoding::Base64,
                    _ => Encoding::QuotedPrintable,
                };

                Ok(HeaderValue::ContentTransferEncoding(encoding))
            },

            _ => {
                Ok(HeaderValue::Unknown(value.to_string()))
            },
        }
    }
}

/// The key value pair of a header.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Header {
    /// The key of the header.
    pub key: String,

    /// The value of the header.
    pub value: HeaderValue,
}

/// Many headers combined together.
#[derive(Debug, Eq, PartialEq)]
pub struct Headers(Vec<Header>);

impl Headers {
    /// Pushes a new header into the collection.
    pub fn push(&mut self, header: Header) {
        self.0.push(header);
    }

    /// Converts the vector of headers into a hashmap.
    pub fn into_hash_map(self) -> HashMap<String, HeaderValue> {
        let mut headers_map = HashMap::new();

        for header in self.0 {
            headers_map.insert(header.key, header.value);
        }

        headers_map
    }
}

impl Header {
    /// Tries to read one header from the reader.
    pub fn try_read<B: BufRead + Clone>(reader: &mut B) -> Result<Header> {
        let mut test_reader = reader.clone();

        let mut read_counter = 0;
        let mut content = String::new();

        test_reader.read_line(&mut content)?;
        read_counter += 1;

        let split = content.split(':').collect::<Vec<_>>();

        if split.len() < 2 {
            return Err(Error::NotAHeader);
        }

        let key = split[0];

        let mut built = vec![split[1].trim().to_owned()];

        loop {

            let mut content = String::new();
            test_reader.read_line(&mut content)?;
            read_counter += 1;
            let ch = content.chars().next();

            match ch {
                None => continue,
                Some(a) if a.is_whitespace() => {
                    built.push(content.trim().to_owned());
                },
                _ => {
                    // We don't want to consume that line
                    read_counter -= 1;
                    break
                },
            }

        }

        for _ in 0 .. read_counter {
            let mut content = String::new();
            reader.read_line(&mut content)?;
        }

        let built = built
            .into_iter()
            .map(|x| x.trim().to_owned())
            .collect::<Vec<_>>().join("\n");

        Ok(Header {
            key: key.to_owned(),
            value: HeaderValue::from_str(key, &built)?,
        })
    }
}

/// A body of a mail.
#[derive(Debug)]
pub struct Body {
    /// The content type of the body.
    content_type: ContentType,

    /// The content of the body.
    content: String,
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// A struct containing the content of an e mail.
pub struct Mail {
    /// The headers of the mail.
    pub headers: HashMap<String, HeaderValue>,

    /// The content of the mail.
    pub plain: Option<String>,

    /// The HTML content of the mail if any.
    pub html: Option<String>,
}

impl Mail {

    /// Parse headers until there are no more.
    pub fn parse_headers<B: BufRead + Clone>(reader: &mut B) -> Result<Headers> {
        let mut headers = vec![];

        loop {
            match Header::try_read(reader) {
                Ok(header) => headers.push(header),
                Err(Error::NotAHeader) => return Ok(Headers(headers)),
                Err(_) => (),
            }
        }
    }

    /// Parse an email from the content received.
    pub fn from_utf8(data: &[u8]) -> Result<Mail> {
        let data = std::str::from_utf8(data)?
            .to_string();

        let mut reader = Cursor::new(data);

        // Turn the headers into a hash map
        let headers = Mail::parse_headers(&mut reader)?
            .into_hash_map();

        let mut current_boundary = match headers.get("Content-Type") {
            Some(HeaderValue::ContentType(ContentType::MultipartAlternative(val))) => Some(val.trim().to_owned()),
            _ => None,
        };

        let mut current_encoding = match headers.get("Content-Transfer-Encoding") {
            Some(HeaderValue::ContentTransferEncoding(e)) => Some(*e),
            _ => None,
        };

        let mut current_content_type = match headers.get("Content-Type") {
            Some(HeaderValue::ContentType(e)) => Some(e.clone()),
            _ => None,
        };

        let mut current_content = String::new();
        let mut bodies = vec![];

        loop {
            let mut content = String::new();
            let exit =  reader.read_line(&mut content)? == 0;

            match current_boundary.clone() {
                Some(ref val) if content.trim().starts_with(val.trim()) => {

                    let headers = Mail::parse_headers(&mut reader)?.into_hash_map();

                    if let Some(HeaderValue::ContentType(ContentType::MultipartAlternative(val))) = headers.get("Content-Type") {
                        current_boundary = Some(val.to_owned());
                    };

                    // Save the current body into the list of bodies
                    if let (Some(content_type), Some(encoding)) = (current_content_type, current_encoding) {
                        bodies.push(Body {
                            content_type,
                            content: encoding.decode(&current_content)?,
                        });
                    }

                    current_content_type = match headers.get("Content-Type") {
                        Some(HeaderValue::ContentType(content_type)) => Some(content_type.clone()),
                        _ => None,
                    };
                    current_encoding = match headers.get("Content-Transfer-Encoding") {
                        Some(HeaderValue::ContentTransferEncoding(encoding)) => Some(*encoding),
                        _ => None,
                    };

                    current_content = String::new();
                    continue;
                },

                _ => (),

            }

            match current_encoding {
                Some(Encoding::QuotedPrintable) => current_content.push_str(&content),
                Some(Encoding::Base64) => current_content.push_str(content.trim()),
                None => (),
            }

            if exit {
                // Save the current body into the list of bodies
                if let (Some(content_type), Some(encoding)) = (current_content_type, current_encoding) {
                    bodies.push(Body {
                        content_type,
                        content: encoding.decode(&current_content)?,
                    });
                }

                break;
            }
        }

        let mut plain = None;
        let mut html = None;

        for body in bodies {
            match body.content_type {
                ContentType::TextPlain => plain = Some(body.content),
                ContentType::TextHtml => html = Some(body.content),
                _ => (),
            }
        }

        Ok(Mail { headers, plain, html })
    }
}

