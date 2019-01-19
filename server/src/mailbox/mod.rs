//! This module contains all the structures for the mail boxes.

use tokio_imap::types::{Response, MailboxDatum};

/// A mail box with emails.
#[derive(Serialize, Deserialize, Debug)]
pub struct Mailbox {
    name: Vec<String>
}

impl Mailbox {
    /// Creates mailbox from a mailbox data from tokio imap.
    pub fn from_data(response: &Response) -> Result<Mailbox, ()> {
        match response {
            Response::MailboxData ( MailboxDatum::List {
                delimiter, name, ..
            }) => Ok(Mailbox {
                name: name.split(delimiter).map(|x| String::from(x)).collect()
            }),

            _ => Err(()),
        }
    }
}
