//! This module contains everything related to the server configuration.

use std::path::Path;
use std::fs::File;
use std::{io, result};
use std::io::Read;

use lettre::{SmtpClient, Transport};
use lettre::smtp::authentication::Credentials;
use lettre_email::Email;
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

    /// The root of the server.
    pub root: String,

    /// The configuration of the database.
    pub database: DatabaseConfig,

    /// The configuration of the mailer.
    pub mailer: Option<Mailer>,
}

impl ServerConfig {

    /// Creates a server config from its attributes.
    pub fn new(root: &str, database: DatabaseConfig, mailer: Option<Mailer>) -> ServerConfig {
        ServerConfig {
            root: String::from(root),
            database,
            mailer,
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
    pub user: String,

    /// The name of the database.
    pub database: String,

    /// The password to connect to the database.
    pub password: String,

    /// The hostname of the database.
    #[serde(default = "localhost")]
    pub hostname: String,
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

/// A structure that will be used to hold a mail configuration.
///
/// This is the mail account chouette will use to send its emails.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mailer {

    /// Whether a mail will be sent for users to activate their accounts.
    pub require_email_validation: bool,

    /// The smtp server of the mail account.
    pub server: String,

    /// The username of the mail account.
    pub username: String,

    /// The password of the mail account.
    pub password: String,

}

impl Mailer {

    /// Creates a new mailer.
    pub fn new(require_email_validation: bool, server: String, username: String, password: String) -> Mailer {
        Mailer {
            require_email_validation,
            server,
            username,
            password,
        }
    }

    /// Uses a mailer to send an email.
    pub fn send_mail(&self, to: &str, subject: String, text: String, html: String) -> Result<()> {

        let email = Email::builder()
            .from(self.username.clone())
            .to(to)
            .subject(subject)
            .alternative(html, text)
            .build()?;

        let mut client = SmtpClient::new_simple(&self.server)
            .expect("Failed to create smtp client")
            .credentials(Credentials::new(self.username.clone(), self.password.clone()))
            .transport();

        client.send(email.into())?;

        Ok(())

    }
}
