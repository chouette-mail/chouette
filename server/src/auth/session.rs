//! Thismodule contains the structures needed to manipulate sessions.

use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::{Error, Result};
use crate::schema::sessions;
use crate::auth::user::User;

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
    pub fn from_secret(key: &str, db: &PgConnection) -> Result<Session> {
        use crate::schema::sessions::dsl::*;
        sessions
            .filter(secret.eq(key))
            .select((id, user_id, secret))
            .first::<Session>(db)
            .map_err(|_| Error::SessionDoesNotExist)
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
