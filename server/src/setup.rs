use std::fs::File;
use std::fmt::Debug;
use std::io::{stdout, stdin, Write};
use std::process::exit;

use colored::*;
use clap::App;
use rpassword::read_password_from_tty;

use chouette::config::{ServerConfig, DatabaseConfig};

fn unwrap<T, E>(param: Result<T, E>) -> T where E: Debug {
    match param {
        Ok(t) => t,
        Err(e) => {
            println!("{} {:?}", "error:".red().bold(), e);
            exit(1);
        },
    }
}

fn flush_stdout() {
    stdout().flush().ok();
}

fn read_password(prompt: &str, default: Option<&str>) -> String {
    loop {
        print!("{}: ", prompt.bold());
        flush_stdout();

        match read_password_from_tty(None) {
            Ok(l) => {
                break match default {
                    Some(x) if l.is_empty() => String::from(x),
                    _ => l,
                }
            }

            Err(_) => {
                println!("{} {}",
                    "error:".red().bold(),
                    "Couldn't read input, please try again.".bold())
            },

        }
    }
}

fn read_input(prompt: &str, default: Option<&str>) -> String {
    loop {
        print!("{}{}: ", prompt.bold(), match default {
            None => String::new(),
            Some(x) => format!(" (default={})", x),
        });

        flush_stdout();
        let mut input = String::new();

        match stdin().read_line(&mut input) {

            Ok(l) => {
                break match (l, default) {
                    (0, Some(x)) => String::from(x),
                    _ => {
                        input.pop();
                        input
                    }
                }
            }

            Err(_) => {
                println!("{} {}",
                    "error:".red().bold(),
                    "Couldn't read input, please try again.".bold())
            },

        }
    }
}

fn main() {

    let _ = App::new("Chouette Mail Setup")
        .version("0.0.0")
        .author("Thomas Forgione <thomas@forgione.fr>")
        .about("A tool to setup the config files for Chouette Mail")
        .get_matches();

    println!("{}", "=== DATABASE SETUP ===".green().bold());

    let db_config = loop {
        let username = read_input("Database username", None);
        let password = read_password("Database user's password", None);
        let database = read_input("Database name", None);
        let host = read_input("Database host", Some("localhost"));

        let db_config = DatabaseConfig::new(username, password, database, host);

        println!("{}", "Trying to connect to the database...".bold());

        match db_config.connect() {
            Ok(_) => {
                println!("{}", "Database connection successful!".green().bold());
                break Some(db_config);
            },
            Err(e) => {
                println!("{} {:?}", "error:".red().bold(), e);
                break None;
            }
        }
    };

    match db_config {
        Some(db_config) => {

            println!("{}", "Saving config files...".bold());
            let server_config = ServerConfig::new("", db_config, None);

            // Write config.toml
            let mut config_toml = unwrap(File::create(chouette::CONFIG_FILE_LOCATION));
            let config = unwrap(toml::to_string(&server_config));
            unwrap(config_toml.write_all(config.as_bytes()));

            // Write .env file
            let mut env = unwrap(File::create(".env"));
            let url = server_config.database.url();
            unwrap(env.write_all(format!("DATABASE_URL={}", url).as_bytes()));

            println!("{}", "=== DATABASE SUCCESSFULLY SETUP ===".green().bold());

        },
        None => {
            println!("{}", "Exiting...".bold());
            exit(1);
        },
    }

}
