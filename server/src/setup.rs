use std::fs::File;
use std::fmt::Debug;
use std::io::{stdout, stdin, Write};
use std::process::exit;

use colored::*;
use clap::App;
use rpassword::read_password_from_tty;

use chouette::CONFIG_FILE_LOCATION;
use chouette::config::{ServerConfig, DatabaseConfig, Mailer};

trait Flatten<T> {
    fn flatten(self) -> Option<T>;
}

impl<T> Flatten<T> for Option<Option<T>> {
    fn flatten(self) -> Option<T> {
        match self {
            Some(Some(x)) => Some(x),
            _ => None
        }
    }
}

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

fn read_password(prompt: &str, default: Option<String>) -> String {
    loop {
        print!("{}: ", prompt.bold());
        flush_stdout();

        match read_password_from_tty(None) {
            Ok(l) => {
                break match default {
                    Some(ref x) if l.is_empty() => x.clone(),
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

fn read_input(prompt: &str, default: Option<String>) -> String {
    loop {
        print!("{}{}: ", prompt.bold(), match default {
            None => String::new(),
            Some(ref x) => format!(" (default={})", x),
        });

        flush_stdout();
        let mut input = String::new();

        match stdin().read_line(&mut input) {

            Ok(l) => {
                break match (l, default) {
                    (l, Some(ref x)) if l <= 1 => x.clone(),
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

fn read_bool(prompt: &str, default: bool) -> bool {
    loop {
        print!("{} {}: ", prompt.bold(), if default { "[Y/n]" } else { "[y/N]" });

        flush_stdout();
        let mut input = String::new();

        match stdin().read_line(&mut input) {

            Ok(l) => {
                match (l, input.as_str()) {
                    (l, _) if l <= 1 => return default,
                    (_, "y\n") | (_, "Y\n")  => {
                        return true;
                    },
                    (_, "n\n") | (_, "N\n") => {
                        return false;
                    },
                    _ => {
                        println!("{} {}",
                            "error:".red().bold(),
                            "Expecting y or n, please try again.".bold());
                    },
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

    println!("{}", "=== WELCOME TO CHOUETTE MAIL CONFIG EDITOR ===".bold().green());
    println!("{}", "This tool will assist you into configuring your chouette mail.".bold());
    println!();

    let config = match ServerConfig::from(CONFIG_FILE_LOCATION) {
        Ok(config) => {
            if read_bool("You already have a valid config file, do you wish to edit it ?", false) {
                Some(config)
            } else {
                println!("{}", "Then, there's nothing tp do...".bold());
                exit(0);
            }
        },
        Err(_) => None,
    };

    println!("{}", "\n--- DATABASE SETUP ---".green().bold());
    println!("{} {}", "INFO:".bold().yellow(), "The database is required to be able to run chouette-mail.".bold());

    let db_config = loop {
        let username = read_input("Database username", config.as_ref().map(|ref x| x.database.user.clone()));
        let password = read_password("Database user's password", config.as_ref().map(|ref x| x.database.password.clone()));
        let database = read_input("Database name", config.as_ref().map(|ref x| x.database.database.clone()));

        let default_host = config
            .as_ref()
            .map(|ref x| x.database.hostname.clone())
            .or(Some(String::from("localhost")));

        let host = read_input("Database host", default_host);

        let db_config = DatabaseConfig::new(username, password, database, host);

        print!("{}", "Trying to connect to the database...".bold());
        flush_stdout();

        match db_config.connect() {
            Ok(_) => {
                println!("{}", " ok!".green().bold());
                break Some(db_config);
            },
            Err(e) => {
                println!();
                println!("{} {:?}", "error:".red().bold(), e);
                if read_bool("Do you want to retry ?", true) {
                    println!();
                } else {
                    println!("{}", "Cannot conitnue without database setup, exiting...".bold());
                    exit(1);
                }
            }
        }
    };

    if db_config.is_some() {
        println!("{}", "--- DATABSE SETUP SUCCESSFUL ---".green().bold());
    }

    println!("{}", "\n--- MAILER SETUP ---".green().bold());
    println!("{} {}", "INFO:".bold().yellow(), "The mailer is the configuration used to automatically send mails.".bold());
    println!("{} {}", "INFO:".bold().yellow(), "It can be used to verify users email addresses.".bold());

    let mailer_config = if read_bool("Do you wish to setup the mailer ?", false)  {
        loop {

            let default_smtp = config
                .as_ref()
                .map(|ref x| x.mailer.as_ref().map(|x| x.server.clone()))
                .flatten();

            let default_username = config
                .as_ref()
                .map(|ref x| x.mailer.as_ref().map(|x| x.username.clone()))
                .flatten();

            let default_password = config
                .as_ref()
                .map(|ref x| x.mailer.as_ref().map(|x| x.password.clone()))
                .flatten();

            let smtp_account = read_input("Address of the SMTP server", default_smtp);
            let username = read_input("Username", default_username);
            let password = read_password("Password", default_password);

            let required = read_bool("Do you want to require email validation ?", true);
            let mailer = Mailer::new(required, smtp_account, username, password);

            if read_bool("Do you want to test this e-mail address parameters ?", false) {

                let target = read_input("Please enter the e-mail to which you want test", None);
                let text = String::from(include_str!("../assets/templates/test.txt"));
                let html = String::from(include_str!("../assets/templates/test.html"));

                print!("{}", "Trying to send an email...");
                flush_stdout();

                let result = mailer.send_mail(&target, String::from("Chouette Mail test"), text, html);

                match result {
                    Ok(_) => {
                        println!("{}", " ok!".bold().green());
                        break Some(mailer);
                    }
                    Err(e) => {
                        println!();
                        println!("{} {:?}", "error:".red().bold(), e);

                        if read_bool("Do you want to retry ?", false) {
                            println!();
                        } else {
                            break None;
                        }
                    }
                }

            } else {
                break Some(mailer);
            }
        }
    } else {
        None
    };

    if mailer_config.is_some() {
        println!("{}", "--- MAILER SETUP SUCCESSFUL ---".green().bold());
    } else {
        println!("{}", "--- MAILER SETUP SKIPPED ---".yellow().bold());
    }

    println!();

    match db_config {
        Some(db_config) => {

            println!("{}", "Generating config files...".bold());
            let server_config = ServerConfig::new("http://localhost:8000", db_config, mailer_config);

            // Write config.toml
            let mut config_toml = unwrap(File::create(chouette::CONFIG_FILE_LOCATION));
            let config = unwrap(toml::to_string(&server_config));
            unwrap(config_toml.write_all(config.as_bytes()));

            // Write .env file
            let mut env = unwrap(File::create(".env"));
            let url = server_config.database.url();
            unwrap(env.write_all(format!("DATABASE_URL={}", url).as_bytes()));

            println!("{}", "=== CHOUETTE SETUP SUCCESSFUL ===".green().bold());

        },
        None => {
            println!("{}", "Exiting...".bold());
            exit(1);
        },
    }

}
