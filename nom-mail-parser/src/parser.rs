//! This module contains all the parsing functions of this crate.

use std::result;
use nom::rest;
use crate::{Result, ContentType, ContentTransferEncoding, Header, Headers, Mail, Body};

/// Parses a boundary appending two dashes in front of it.
fn parse_boundary(input: &[u8]) -> Vec<u8> {
    let mut real_boundary = vec!['-' as u8, '-' as u8];
    real_boundary.extend_from_slice(input);
    real_boundary
}

/// Converts a Vec<u8> into a string assuming Utf8 encoded.
fn vec_u8_to_string(input: Vec<u8>) -> result::Result<String, std::str::Utf8Error> {
    Ok(std::str::from_utf8(&input)?.trim().to_string())
}

/// Parses a content transfer encoding.
fn parse_content_transfer_encoding(input: &[u8]) -> result::Result<ContentTransferEncoding, ()> {
    match input {
        b"quoted-printable" => Ok(ContentTransferEncoding::QuotedPrintable),
        b"base64" => Ok(ContentTransferEncoding::Base64),
        _ => Err(()),
    }
}

/// Parses the content of a header value, managing correctly the new lines.
named!(header_value_aux<&[u8], Vec<u8>>,
   alt!(
        terminated!(take_until_and_consume!("\r\n"), peek!(is_not!(" \t"))) => { |x: &[u8]| {
            let mut ret = vec![];
            ret.extend_from_slice(x);
            ret
        }} |
        pair!(take_until_and_consume!("\r\n"), header_value_aux) => { |(x, mut y): (&[u8], Vec<u8>)| {
            let mut ret = vec![];
            ret.extend_from_slice(x);
            ret.append(&mut y);
            ret
        }}
   )
);

/// Parses the content of a header value, returning a String.
named!(header_value<&[u8], String>,
    map_res!(header_value_aux, vec_u8_to_string)
);

/// Parses the subject header of a mail.
named!(subject<&[u8], String>,
    preceded!(tag_no_case!("Subject:"), header_value)
);

/// Parses the date header of a mail.
named!(date<&[u8], String>,
    preceded!(tag_no_case!("Date:"), header_value)
);

/// Parses the from header of a mail.
named!(from<&[u8], String>,
    preceded!(tag_no_case!("From:"), header_value)
);

/// Parses an unknown header of a mail.
named!(unknown_header<&[u8], String>,
    preceded!(peek!(is_not!("\r\n")), header_value)
);

/// Parses a multipart alternative of a mail, and returns its boundary.
named!(multipart_alternative<&[u8], Vec<u8>>,
    map!(
        terminated!(
            preceded!(
                tag!("multipart/alternative; boundary="),
                delimited!(char!('"'), take_until!("\""), char!('"'))
            ),
            tag!("\r\n")
        ),
        parse_boundary
    )
);

/// Parses the content type of a mail.
named!(content_type<&[u8], ContentType>, preceded!(tag!("Content-Type: "), alt!(
    multipart_alternative => { ContentType::MultipartAlternative } |
    preceded!(tag!("text/plain"), take_until_and_consume!("\r\n")) => { |_| ContentType::TextPlain } |
    preceded!(tag!("text/html"), take_until_and_consume!("\r\n")) => { |_| ContentType::TextHtml }
)));

/// Parses the content transfer encoding of a mail.
named!(content_transfer_encoding<&[u8], ContentTransferEncoding>,
    map_res!(
        preceded!(
            tag_no_case!("Content-Transfer-Encoding: "),
            take_until_and_consume!("\r\n")
        ), parse_content_transfer_encoding
    )
);

/// Parses a header of a mail.
named!(header<Header>, alt!(
    subject => { Header::Subject }
    | date => { Header::Date }
    | from => { Header::From }
    | content_type => { Header::ContentType }
    | content_transfer_encoding => { Header::ContentTransferEncoding }
    | unknown_header => { Header::Unknown }
));

/// Parses all the headers of a mail.
named!(headers<&[u8], Headers >,
    map!(many0!(header), Headers)
);

/// Parses a single body mail.
named!(parse_mail<&[u8], Mail>,
    do_parse!(
        h: headers >>
        tag!("\r\n") >>
        content: rest >>
        ({
            Mail {
                headers: h,
                body: Body::Content(std::str::from_utf8(content).unwrap().to_string()),
            }
        })
    )
);

/// Parses a multi body mail, containing only single bodies.
named!(parse_multi_mail<&[u8], Mail>,
    do_parse!(
        h: headers >>
        _dummy: take_until_and_consume!(&h.boundary().unwrap()[..]) >>
        tag!("\r\n") >>
        printable: take_until_and_consume!(&h.boundary().unwrap()[..]) >>
        tag!("\r\n") >>
        html: take_until_and_consume!(&h.boundary().unwrap()[..]) >>
        ({
            Mail {
                headers: h,
                body: Body::Multi(vec![parse_mail(printable)?.1, parse_mail(html)?.1]),
            }
        })
    )
);

/// Parses a mail.
pub fn parse(bytes: &[u8]) -> Result<Mail> {
    Ok(parse_multi_mail(bytes)?.1)
}

/// Parses only the headers of a mail.
///
/// This is useful if you make an IMAP request that doesn't fetch the body of a mail but only the
/// headers.
pub fn parse_headers(bytes: &[u8]) -> Result<Headers> {
    Ok(headers(bytes)?.1)
}
