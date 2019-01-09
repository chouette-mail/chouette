//! This module contains everything related to the authentication of users.

use rand::RngCore;
use rand::rngs::OsRng;
use bcrypt::{DEFAULT_COST, hash, verify};
use diesel::RunQueryDsl;
use diesel::pg::PgConnection;
use crate::schema::{users, imap_accounts};
use crate::config::DatabaseError;

/// A user of chouette.
#[derive(Identifiable, Queryable, PartialEq, Debug)]
pub struct User {
    /// The id of the user.
    pub id: i32,

    /// The username of the user.
    pub username: String,

    /// The email of the user.
    pub email: String,

    /// The BCrypt hash of the password of the user.
    pub hashed_password: String,

    /// Whether the user is activated or not.
    pub activated: bool,

    /// The activation key of the user if it is not active.
    pub activation_key: Option<Vec<u8>>,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
/// A user that is stored into the database yet.
pub struct NewUser {
    /// The username of the user.
    pub username: String,

    /// The email of the new user.
    pub email: String,

    /// The BCrypt hashed password of the new user.
    pub hashed_password: String,

    /// Whether the new user is automatically activated or not.
    pub activated: bool,

    /// The activation key of the new user.
    pub activation_key: Option<Vec<u8>>,
}

impl User {
    /// Creates a new user.
    pub fn create(username: &str, email: &str, password: &str) -> Result<NewUser, ()> {

        // Hash the password
        let hashed_password = hash(&password, DEFAULT_COST)
            .expect("Couldn't hash password");

        // Generate the activation key
        let mut rng = OsRng::new().unwrap();
        let mut activation_key = vec![0u8; 20];
        rng.fill_bytes(&mut activation_key);

        Ok(NewUser {
            username: String::from(username),
            email: String::from(email),
            hashed_password: String::from(hashed_password),
            activated: false,
            activation_key: Some(activation_key),
        })

    }

    /// Adds a new imap server for the user.
    pub fn new_imap_account(&self, server: &str, username: &str) -> Result<NewImapAccount, ()> {
        Ok(NewImapAccount {
            user_id: self.id,
            server: String::from(server),
            username: String::from(username),
        })
    }
}

impl NewUser {
    /// Saves the new user into the database.
    pub fn save(&self, database: PgConnection) -> Result<User, DatabaseError> {
        Ok(diesel::insert_into(users::table)
            .values(self)
            .get_result(&database)?)

    }
}

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
}
