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

/// Parses a content transfer encoding.
fn parse_content_transfer_encoding(input: &[u8]) -> result::Result<ContentTransferEncoding, ()> {
    match input {
        b"quoted-printable" => Ok(ContentTransferEncoding::QuotedPrintable),
        b"base64" => Ok(ContentTransferEncoding::Base64),
        _ => Err(()),
    }
}

/// Decodes a base64 encoded string.
named!(decode_base64<&[u8], String>,
    map!(
        pair!(separated_list!(is_a!(" \t"), alt!(
            preceded!(tag!("=?UTF-8?B?"), take_until_and_consume!("?="))
        )), tag!("\r\n")),
        |(x, _)| {
            let mut decoded = String::new();
            for i in x {
                decoded.push_str(std::str::from_utf8(&base64::decode(i).unwrap()).unwrap());
            }
            decoded
        }
    )
);

/// Convers a &[u8] to a string depending on the encoding.
named!(u8_to_string<&[u8], String>,
    alt!(
        preceded!(peek!(tag!("=?UTF-8?B?")), decode_base64) |
        map!(map_res!(take_until_and_consume!("\r\n"), { std::str::from_utf8 }), { str::to_string })
    )
);

/// Parses a header value.
named!(header_value<&[u8], String>,
    map!(
        pair!(
            many0!(
                terminated!(u8_to_string, is_a!(" \t"))
            ),
            terminated!(u8_to_string, peek!(is_not!(" \t")))
        ),
        |(mut x, y): (Vec<String>, String)| {
            x.push(y);
            let ret = x.join("");
            ret
        }
    )
);

/// Parses the subject header of a mail.
named!(subject<&[u8], String>,
    preceded!(tag_no_case!("Subject: "), header_value)
);

/// Parses the date header of a mail.
named!(date<&[u8], String>,
    preceded!(tag_no_case!("Date: "), header_value)
);

/// Parses the from header of a mail.
named!(from<&[u8], String>,
    preceded!(tag_no_case!("From: "), header_value)
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
                preceded!(preceded!(tag!("multipart/"), take_until_and_consume!("; ")), tag!("boundary=")),
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

/// Parses a mail.
named!(parse_mail<&[u8], Mail>,
    do_parse!(
        h: headers >>

        // Multipart part
        _dummy: cond!(h.boundary().is_some(), take_until_and_consume!({
            &h.boundary().unwrap()[..]
        })) >>
        printable: cond!(h.boundary().is_some(), many0!(preceded!(tag!("\r\n"), take_until_and_consume!(&h.boundary().unwrap()[..])))) >>

        // Single part
        cond!(h.boundary().is_none(), tag!("\r\n")) >>
        content: cond!(h.boundary().is_none(), rest) >>
        (
            if let Some(printable) = printable {
                Mail {
                    headers: h,
                    body: Body::Multi(printable.iter().map(|x| {
                        parse_mail(x).unwrap().1
                    }).collect()),
                }
            } else {
                Mail {
                    headers: h,
                    body: Body::Content(std::str::from_utf8(content.unwrap()).unwrap().to_string()),
                }
            }
        )
    )
);

/// Parses a mail.
pub fn parse(bytes: &[u8]) -> Result<Mail> {
    Ok(parse_mail(bytes)?.1)
}

/// Parses only the headers of a mail.
///
/// This is useful if you make an IMAP request that doesn't fetch the body of a mail but only the
/// headers.
pub fn parse_headers(bytes: &[u8]) -> Result<Headers> {
    Ok(headers(bytes)?.1)
}
