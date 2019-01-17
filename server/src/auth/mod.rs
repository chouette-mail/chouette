//! This module contains everything related to the authentication of users.

use rand::Rng;
use rand::rngs::OsRng;
use rand::distributions::Alphanumeric;
use bcrypt::{DEFAULT_COST, hash, verify};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::pg::PgConnection;
use crate::schema::{users, imap_accounts, sessions};
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

/// A user that is stored into the database yet.
#[derive(Debug, Insertable)]
#[table_name = "users"]
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

    /// Creates or updates a session for a user that has been authenticated.
    pub fn save_session(&self, db: &PgConnection) -> Result<Session, DatabaseError> {
        // Generate the secret
        let mut rng = OsRng::new().unwrap();
        let secret = rng
            .sample_iter(&Alphanumeric)
            .take(40)
            .collect::<String>();

        let session = NewSession {
            user_id: self.id,
            secret,
        };

        Ok(diesel::insert_into(sessions::table)
            .values(&session)
            .get_result(db)?)
    }

    /// Adds a new imap server for the user.
    pub fn new_imap_account(&self, server: &str, username: &str, password: &str) -> NewImapAccount {
        ImapAccount::new(self.id, server, username, password)
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
    pub fn save(&self, db: &PgConnection) -> Result<ImapAccount, DatabaseError> {
        Ok(diesel::insert_into(imap_accounts::table)
            .values(self)
            .get_result(db)?)
    }
}

/// A session that belongs to a user.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
pub struct Session {
    /// The id of the session.
    pub id: i32,

    /// The owned of the session.
    pub user_id: i32,

    /// The secret id of the session.
    pub secret: String,
}

impl Session {
    /// Finds a session in the database from its secret key.
    ///
    /// Returns none if no session was found.
    pub fn from_secret(key: &str, db: &PgConnection) -> Option<Session> {
        use crate::schema::sessions::dsl::*;
        sessions
            .filter(secret.eq(key))
            .select((id, user_id, secret))
            .first::<Session>(db)
            .ok()
    }
}

/// A new session not stored in the database yet.
#[derive(Debug, Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    /// The owner of the session.
    pub user_id: i32,

    /// The secret id of the session.
    pub secret: String,
}
