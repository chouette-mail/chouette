//! This module contains the structures to manipulate imap accounts.

use diesel::prelude::*;

use futures_state_stream::StateStream;
use tokio::prelude::*;
use tokio::prelude::future::*;
use tokio_imap::client::{ImapClient, ImapConnectFuture, TlsClient};
use imap_proto::builders::command::{Command, CommandBuilder};

use crate::{Error, Result};
use crate::mailbox::Mailbox;
use crate::schema::imap_accounts;
use crate::schema::smtp_accounts;
use crate::auth::user::User;

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
            pub fn new(user_id: i32, server: &str, username: &str, password: &str) -> $insertable_struct {
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
    pub fn test(server: &str, username: &str, password: &str) -> Result<impl Future> {
        let username = String::from(username);
        let password = String::from(password);
        let command = CommandBuilder::login(&username, &password);

        Ok(TlsClient::connect(server)?
            .map_err(Into::<Error>::into)
            .and_then(|(_, connection)| {
                connection.call(command).collect()
                    .map_err(Into::<Error>::into)
            }))
    }

    /// Fetches the mailboxes of the imap account.
    pub fn fetch_mailboxes(self) -> impl Future {

        let username = self.username.clone();
        let username_bis = self.username.clone();

        TlsClient::connect(&self.server)
            .into_future()
            .and_then(|x| x)
            .and_then(move |connection| {
                info!("Connected to the imap server {}", self.server);

                let command = CommandBuilder::login(&self.username, &self.password);
                connection.1.call(command).collect()
            })
        .and_then(move |(_, connection)| {
            info!("Fetching the mailboxes for user {}", username);
            let command = CommandBuilder::list("", "*");
            connection.call(command).collect()
        })
        .and_then(move |(response, _)| {
            info!("Fetched the mailboxes for user {}", username_bis);

            let mut mailboxes = vec![];
            for data in response {
                if let Ok(mailbox) = Mailbox::from_data(&data.parsed()) {
                    mailboxes.push(mailbox);
                }
            }

            Ok(mailboxes)
        })
    }

}
