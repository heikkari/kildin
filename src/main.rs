#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

pub mod helpers;
pub mod database;
pub mod server;
pub mod proxy_checker;
pub mod ratelimit_updater;

// crate
use crate::helpers::logger::{Level, Logger};
use crate::helpers::config::{Config, ProxyCheckerSettings};
use crate::ratelimit_updater::RatelimitUpdater;
use crate::proxy_checker::ProxyChecker;
use crate::database::managers::ManagerAuth;
use crate::database::ratelimited::RateLimited;
use crate::database::proxies::Proxies;
use crate::helpers::types;

// std
use std::thread;
use std::time::Duration;
use std::env::args;
use std::path::Path;
use std::fs;

// rusqlite
use rusqlite::Connection;

fn start_proxy_checker(pcs: ProxyCheckerSettings, logger: Logger) {
    logger.log(Level::Info, "The proxy checker and rate limit updater are starting!");

    // create structs
    let mut pc = ProxyChecker::new(pcs.clone(), logger.clone());
    let mut ru = RatelimitUpdater::new(logger.clone());
    let dur = Duration::from_secs(pcs.interval);

    let cloned_logger = logger.clone();

    thread::spawn(move || {
        loop {
            let logger = cloned_logger.clone();

            match pc.update() {
                Ok(b) => if b { logger.log(Level::Info, "Successfully checked health of/updated the proxies!") },
                Err(why) => logger.log(Level::Error,
                    &format!("ProxyChecker/Error: {}", why))
            }
        }
    });

    thread::spawn(move || {
        loop {
            let logger = logger.clone();

            match ru.update() {
                Ok(_) => logger.log(Level::Info, "Successfully checked up on the rate limited proxies!"),
                Err(why) => logger.log(Level::Error,
                    &format!("RatelimitUpdater/Error: {}", why))
            }

            thread::sleep(dur);
        }
    });
}

fn load_config() -> Config {
    let args = args().collect::<Vec<String>>();
    let contents = fs::read_to_string(args[1].clone()).expect("Something went wrong with the file");
    Config::from(&contents).expect("Couldn't read config")
}

fn connect_to_database() -> Result<Connection, types::AnyError> {
    let config = load_config();
    let conn = Connection::open(Path::new(&config.general.database_path))?;
    Ok(conn)
}

fn setup_db() {
    Proxies::create(connect_to_database().unwrap());
    ManagerAuth::create(connect_to_database().unwrap());
    RateLimited::create(connect_to_database().unwrap());
}

fn main() {
    let logger = Logger::new();

    // report launch
    logger.log(Level::Info, "Currently running: Kildin v1.0");
    logger.log(Level::Info, "Kildin is starting.");

    // connect to database
    let config = load_config();
    setup_db(); // setup db in case it isn't properly created
    logger.log(Level::Info, "Database checked!");

    // start proxy checker
    start_proxy_checker(config.proxy_settings, logger.clone());
    logger.log(Level::Info, "The proxy checker has been started!");

    // start server
    server::start();
}
