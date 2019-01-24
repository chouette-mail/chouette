//! This module contains the structures to manipulate imap accounts.

use diesel::prelude::*;

use crate::Result;
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

