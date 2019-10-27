#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate scrypt;
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_uri = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_uri)
        .expect(&format!("Error connecting to {}", database_uri))
}
