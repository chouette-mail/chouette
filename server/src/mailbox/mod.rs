//! This module contains all the structures for the mail boxes.

pub mod mail;

use imap::types::Name;

#[derive(Serialize, Deserialize)]
/// A mailbox from an IMAP account.
pub struct Mailbox {
    /// The parts of the name of the mailbox.
    name: Vec<String>,
}

impl From<&Name> for Mailbox {
    fn from(name: &Name) -> Mailbox {
        Mailbox {
            name: if let Some(delimiter) = name.delimiter() {
                name.name().split(delimiter).map(String::from).collect()
            } else {
                vec![String::from(name.name())]
            }
        }
    }
}
