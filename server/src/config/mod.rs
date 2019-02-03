//! This module contains everything related to the server configuration.

use std::path::Path;
use std::fs::File;
use std::{io, result};
use std::io::Read;

use serde_derive::{Serialize, Deserialize};
use diesel::connection::Connection;
use diesel::pg::PgConnection;

use crate::Result;

/// Returns the string localhost.
fn localhost() -> String {
    String::from("localhost")
}

/// The errors that can occur during the configuration of the server.
#[derive(Debug)]
pub enum Error {
    /// An IoError occured while reading the config file.
    IoError(io::Error),

    /// An error occured while parsing the toml file.
    TomlError(toml::de::Error),

}

impl_from_error!(Error, Error::IoError, io::Error);
impl_from_error!(Error, Error::TomlError, toml::de::Error);

/// The server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// The configuration of the database.
    pub database: DatabaseConfig,
}

impl ServerConfig {

    /// Creates a server config from its attributes.
    pub fn new(database: DatabaseConfig) -> ServerConfig {
        ServerConfig {
            database,
        }
    }

    /// Creates a new server config from a path to a toml file.
    pub fn from<P: AsRef<Path>>(path: P) -> result::Result<ServerConfig, Error> {
        let mut content = String::new();
        let mut toml_file = File::open(path)?;
        toml_file.read_to_string(&mut content)?;
        Ok(toml::from_str(&content)?)
    }
}

/// The configuration of a database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// The user of the database.
    user: String,

    /// The name of the database.
    database: String,

    /// The password to connect to the database.
    password: String,

    /// The hostname of the database.
    #[serde(default = "localhost")]
    hostname: String,
}

impl DatabaseConfig {

    /// Creates a database config from its attributes.
    pub fn new(user: String, password: String, database: String, hostname: String) -> DatabaseConfig {
        DatabaseConfig {
            user,
            password,
            database,
            hostname,
        }
    }

    /// Returns the url to connect to the database.
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}/{}",
            self.user,
            self.password,
            self.hostname,
            self.database
        )
    }

    /// Returns a connection to the database.
    pub fn connect(&self) -> Result<PgConnection> {
        Ok(PgConnection::establish(&self.url())?)
    }
}

/// An error occured while trying to manipulate the database.
#[derive(Debug)]
pub enum DatabaseError {
    /// Couldn't connect to the database.
    ConnectionError(diesel::ConnectionError),

    /// Error while running a database request.
    RequestError(diesel::result::Error),
}

impl_from_error!(DatabaseError, DatabaseError::ConnectionError, diesel::ConnectionError);
impl_from_error!(DatabaseError, DatabaseError::RequestError, diesel::result::Error);
