#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
extern crate env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::io::stdout;
use std::path::Path;

use diesel::prelude::*;
use diesel::r2d2;
use diesel::result::Error;
use diesel_migrations::{
    connection::MigrationConnection, run_pending_migrations_in_directory, RunMigrationsError,
};

/// Type of a pooled SQLite connection manager.
pub type SqliteConnectionManager = r2d2::ConnectionManager<SqliteConnection>;

/// Type for a SQLite connection pool.
pub type SqliteConnectionPool = r2d2::Pool<SqliteConnectionManager>;

/// Result formats for verses.
pub enum VerseFormat {
    /// Literal HTML.
    HTML,
    /// Plain text with no special formatting.
    PlainText,
}

/// Error type that for the Bible.rs application.
#[derive(Fail, Debug)]
pub enum BiblersError {
    #[fail(display = "'{}' was not found.", book)]
    BookNotFound { book: String },

    #[fail(display = "There was a connection pool error.",)]
    ConnectionPoolError { root_cause: String },

    #[fail(
        display = "There was a database error. Root cause: {:?}.",
        root_cause
    )]
    DatabaseError { root_cause: Error },

    #[fail(
        display = "There was a database migration error. Root cause: {:?}.",
        root_cause
    )]
    DatabaseMigrationError { root_cause: RunMigrationsError },

    #[fail(display = "'{}' is not a valid Bible reference.", reference)]
    InvalidReference { reference: String },

    #[fail(display = "There was an error rendering the HTML page.")]
    TemplateError,
}

/// Builds a SQLite connection bool with the given URL.
pub fn build_pool(db_url: &str) -> SqliteConnectionPool {
    r2d2::Pool::builder()
        .max_size(15)
        .build(SqliteConnectionManager::new(db_url))
        .unwrap()
}

/// Establishes a non-pooled SQLite connection.
pub fn establish_connection(db_url: &str) -> SqliteConnection {
    SqliteConnection::establish(db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}

/// Run any pending Diesel migrations.
pub fn run_migrations<Conn>(conn: &Conn) -> Result<(), BiblersError>
where
    Conn: MigrationConnection,
{
    let dir = Path::new("./db/migrations");
    run_pending_migrations_in_directory(conn, &dir, &mut stdout())
        .map_err(|e| BiblersError::DatabaseMigrationError { root_cause: e })
}

pub mod models;
mod schema;
pub mod sword_drill;
