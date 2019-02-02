//! This module contains the structures to manipulate imap accounts.

use std::net::TcpStream;

use diesel::prelude::*;

use native_tls::TlsStream;
use imap::Session;
use nom_mail_parser::parse_headers;

use crate::{Error, Result};
use crate::schema::imap_accounts;
use crate::schema::smtp_accounts;
use crate::auth::user::User;
use crate::mailbox::Mailbox;

macro_rules! make_account {
    ($queryable_struct: ident, $insertable_struct: ident, $table: expr, $table_name: expr) => {
        #[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
        #[belongs_to(User)]
        /// An account to connect to a mail server.
        pub struct $queryable_struct {
            /// The id of the account.
            pub id: i32,

            /// The owner of the account.
            pub user_id: i32,

            /// The server address.
            pub server: String,

            /// The username to log to the server.
            pub username: String,

            /// The password to log to the server.
            ///
            /// FIXME: For the moment, the password is stored in clear.

            // A potential solution would be to encrypt the password with the user's real password, that
            // way we would be able to retrieve the password but we wouldn't be abl to retrieve it
            // without, so we could potentially be safe even if the db leaks.
            pub password: String,
        }

        impl $queryable_struct {
            /// Creates a new account that is not stored in the db yet.
            pub fn create(user_id: i32, server: &str, username: &str, password: &str) -> $insertable_struct {
                $insertable_struct {
                    user_id,
                    server: String::from(server),
                    username: String::from(username),
                    password: String::from(password),
                }
            }
        }

        /// A new account not stored into the database yet.
        #[derive(Debug, Insertable)]
        #[table_name = $table_name]
        pub struct $insertable_struct {
            /// The owner of the account.
            pub user_id: i32,

            /// The server address.
            pub server: String,

            /// The username to log to the server.
            pub username: String,

            /// The password to log to the server.
            pub password: String,
        }


        impl $insertable_struct {
            /// Saves a new account into the database and returns the corresponding account.
            pub fn save(&self, db: &PgConnection) -> Result<$queryable_struct> {
                Ok(diesel::insert_into($table)
                   .values(self)
                   .get_result(db)?)
            }
        }

    }
}

make_account!(ImapAccount, NewImapAccount, imap_accounts::table, "imap_accounts");
make_account!(SmtpAccount, NewSmtpAccount, smtp_accounts::table, "smtp_accounts");

impl ImapAccount {

    /// Tries to connect to the imap account.
    pub fn test(server: &str, username: &str, password: &str) -> Result<Session<TlsStream<TcpStream>>> {
        let tls = native_tls::TlsConnector::builder().build()?;
        let client = imap::connect((server, 993), server, &tls)?;
        Ok(client.login(username, password)?)
    }

    /// Logs in the imap account and return the session.
    pub fn login(&self) -> Result<Session<TlsStream<TcpStream>>> {
        ImapAccount::test(&self.server, &self.username, &self.password)
    }

    /// Fetches the mailboxes of the imap account.
    pub fn fetch_mailboxes(&self) -> Result<Vec<Mailbox>> {
        let mut session = self.login()?;

        Ok(session.list(Some("/"), Some("*"))?
            .into_iter()
            .map(Mailbox::from)
            .collect())

    }

    /// Fetches all the imap accounts of the user with the corresponding id.
    pub fn from_user_id(user: i32, connection: &PgConnection) -> Result<Vec<ImapAccount>> {
        use crate::schema::imap_accounts::dsl::*;
        Ok(imap_accounts
            .filter(user_id.eq(user))
            .select((id, user_id, server, username, password))
            .get_results::<ImapAccount>(connection)
            .map_err(Into::<Error>::into)?)
    }

    /// Fetches all the subjects of mails in a range.
    pub fn fetch_subjects(&self, mailbox: &str, start: usize, end: usize) -> Result<Vec<String>> {
        let mut session = self.login()?;
        session.select(mailbox)?;
        let mut subjects = vec![];

        for i in start .. end {

            if let Ok(message) = session.fetch(i.to_string(), "(FLAGS RFC822.HEADER)") {
                let message = if let Some(m) = message.iter().next() {
                    m
                } else {
                    continue;
                };

                let headers = message.header().unwrap_or(&[]);

                let headers = match parse_headers(headers) {
                    Ok(h) => h,
                    Err(_) => continue,
                };

                if let Some(subject) = headers.subject() {
                    subjects.push(subject.clone());
                }
            }
        }

        Ok(subjects)
    }
}
