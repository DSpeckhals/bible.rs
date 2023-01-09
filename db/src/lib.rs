#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use std::path::Path;

use diesel::prelude::*;
use diesel::r2d2;
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use thiserror::Error;

use crate::models::Book;

/// Type of a pooled SQLite connection manager.
pub type SqliteConnectionManager = r2d2::ConnectionManager<SqliteConnection>;

/// Type for a SQLite connection pool.
pub type SqliteConnectionPool = r2d2::Pool<SqliteConnectionManager>;

pub type DbConnection = SqliteConnection;

/// Result formats for verses.
pub enum VerseFormat {
    /// Literal HTML.
    Html,
    /// Plain text with no special formatting.
    PlainText,
}

#[derive(Clone, Error, Debug)]
pub enum DbError {
    #[error("'{}' was not found.", book)]
    BookNotFound { book: String },

    #[error("There was a connection pool error.")]
    ConnectionPool { cause: String },

    #[error("There was a database error. Root cause: {:?}.", cause)]
    Other { cause: String },

    #[error("There was a database migration error. Root cause: {:?}.", cause)]
    Migration { cause: String },

    #[error("'{}' is not a valid Bible reference.", reference)]
    InvalidReference { reference: String },
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
pub fn run_migrations(conn: &mut SqliteConnection) -> Result<(), DbError> {
    let dir = Path::new("./db/migrations");
    let source = FileBasedMigrations::find_migrations_directory_in_path(dir).map_err(|e| {
        DbError::Migration {
            cause: e.to_string(),
        }
    })?;
    conn.run_pending_migrations(source)
        .map(|_| ())
        .map_err(|e| DbError::Migration {
            cause: e.to_string(),
        })
}

pub fn prefetch_books(conn: &mut SqliteConnection) -> Result<Vec<Book>, DbError> {
    use crate::schema::books;

    books::table.load(conn).map_err(|e| DbError::Other {
        cause: format!("Could not preload book data from database. Cause: {e}"),
    })
}

pub mod models;
mod schema;
mod sword_drill;

pub use sword_drill::{SwordDrill, SwordDrillable};
