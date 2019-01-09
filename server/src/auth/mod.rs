//! This module contains everything related to the authentication of users.

use rand::Rng;
use rand::rngs::OsRng;
use rand::distributions::Alphanumeric;
use bcrypt::{DEFAULT_COST, hash, verify};
use diesel::prelude::*;
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
    pub activation_key: Option<String>,
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
    pub activation_key: Option<String>,
}

impl User {
    /// Creates a new user.
    pub fn create(username: &str, email: &str, password: &str) -> Result<NewUser, ()> {

        // Hash the password
        let hashed_password = hash(&password, DEFAULT_COST)
            .expect("Couldn't hash password");

        // Generate the activation key
        let mut rng = OsRng::new().unwrap();
        let activation_key = rng
            .sample_iter(&Alphanumeric)
            .take(40)
            .collect::<String>();

        Ok(NewUser {
            username: String::from(username),
            email: String::from(email),
            hashed_password: String::from(hashed_password),
            activated: false,
            activation_key: Some(activation_key),
        })

    }

    /// Authenticates a user from its username and password.
    pub fn authenticate(auth_username: &str, auth_password: &str, db: &PgConnection) -> Option<User> {
        use crate::schema::users::dsl::*;

        let user = users
            .filter(username.eq(auth_username))
            // .filter(activated.eq(true))
            .select((id, username, email, hashed_password, activated, activation_key))
            .first::<User>(db);

        let user = match user {
            Ok(user) => user,
            Err(e) => {
                error!("Failed to execute request from db: {:?}", e);
                return None;
            },
        };

        match bcrypt::verify(&auth_password, &user.hashed_password) {
            Ok(true) => Some(user),
            Ok(false) => None,
            Err(e) => {
                error!("Failed to check password for user {:?}: {:?}", user, e);
                None
            }
        }
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
    pub fn save(&self, database: &PgConnection) -> Result<User, DatabaseError> {
        Ok(diesel::insert_into(users::table)
            .values(self)
            .get_result(database)?)

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
