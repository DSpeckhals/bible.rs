pub mod controllers;
pub mod models;
pub mod reference;
pub mod schema;
pub mod sword_drill;

extern crate actix_web;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate handlebars;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate url;
extern crate url_serde;

use std::env;

use diesel::prelude::*;
use diesel::r2d2;
use diesel::result::Error;
use dotenv::dotenv;
use handlebars::Handlebars;

pub type SqliteConnectionManager = r2d2::ConnectionManager<SqliteConnection>;
pub type SqliteConnectionPool = r2d2::Pool<SqliteConnectionManager>;

pub struct ServerState {
    pub db: SqliteConnectionPool,
    pub template: Handlebars,
}

#[derive(Fail, Debug)]
pub enum ReceptusError {
    #[fail(display = "'{}' was not found.", book)]
    BookNotFound { book: String },

    #[fail(display = "There was a connection pool error.",)]
    ConnectionPoolError { root_cause: String },

    #[fail(
        display = "There was a database error. Root cause: {:?}.",
        root_cause
    )]
    DatabaseError { root_cause: Error },

    #[fail(display = "'{}' is not a valid Bible reference.", reference)]
    InvalidReference { reference: String },

    #[fail(display = "There was an error rendering the HTML page.")]
    TemplateError,
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
