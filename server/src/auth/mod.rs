//! This module contains everything related to the authentication of users.

use rand::RngCore;
use rand::rngs::OsRng;
use bcrypt::{DEFAULT_COST, hash, verify};
use diesel::RunQueryDsl;
use diesel::pg::PgConnection;
use crate::schema::auth_user;
use crate::config::DatabaseError;

/// A user of chouette.
#[derive(Debug, Queryable)]
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
#[table_name="auth_user"]
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
}

impl NewUser {
    /// Saves the new user into the database.
    pub fn save(&self, database: PgConnection) -> Result<User, DatabaseError> {
        Ok(diesel::insert_into(auth_user::table)
            .values(self)
            .get_result(&database)?)

    }
}
