//! This module contains the structures to manipulate imap accounts.

use diesel::prelude::*;

use crate::Result;
use crate::schema::imap_accounts;
use crate::auth::user::User;

/// An imap account that is owned by a user.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
pub struct ImapAccount {
    /// The id of the imap account.
    pub id: i32,

    /// The owner of the imap account.
    pub user_id: i32,

    /// The imap server address.
    pub server: String,

    /// The username to log to the imap server.
    pub username: String,

    /// The password to log to the imap server.
    ///
    /// FIXME: For the moment, the password is stored in clear.

    // A potential solution would be to encrypt the password with the user's real password, that
    // way we would be able to retrieve the imap password but we wouldn't be abl to retrieve it
    // without, so we could potentially be safe even if the db leaks.
    pub password: String,
}

impl ImapAccount {
    /// Creates a new imap account that is not stored in the db yet.
    pub fn new(user_id: i32, server: &str, username: &str, password: &str) -> NewImapAccount {
        NewImapAccount {
            user_id,
            server: String::from(server),
            username: String::from(username),
            password: String::from(password),
        }
    }
}

/// A new imap account not stored into the database yet.
#[derive(Debug, Insertable)]
#[table_name = "imap_accounts"]
pub struct NewImapAccount {
    /// The owner of the imap account.
    pub user_id: i32,

    /// The imap server address.
    pub server: String,

    /// The username to log to the imap server.
    pub username: String,

    /// The password to log to the imap server.
    pub password: String,
}

impl NewImapAccount {
    /// Saves a new imap account into the database and returns the corresponding imap account.
    pub fn save(&self, db: &PgConnection) -> Result<ImapAccount> {
        Ok(diesel::insert_into(imap_accounts::table)
            .values(self)
            .get_result(db)?)
    }
}


