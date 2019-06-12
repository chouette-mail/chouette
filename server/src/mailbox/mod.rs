//! This module contains all the structures for the mail boxes.

use imap::types::Name;

/// A mailbox from an IMAP account.
#[derive(Serialize, Deserialize, Debug)]
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
