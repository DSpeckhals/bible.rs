//! Actors for actx sync arbiters
//!
//! Each actor implements a message and its handler, and generalizes fine-grained
//! errors returned from the database layer.
use actix_web::actix::*;
use log::error;

use db::models::*;
use db::sword_drill;
use db::SqliteConnectionPool;
use db::{DbError, VerseFormat};

use crate::error::Error;

/// Maps a specific database error to a generic one, and log.
fn map_db_err(e: DbError) -> Error {
    error!("{}", e);
    match e {
        DbError::BookNotFound { book } => Error::BookNotFound { book },
        DbError::InvalidReference { reference } => Error::InvalidReference { reference },
        _ => Error::Db,
    }
}

/// Sync arbiter for executing SQLite database interations with pooled connections.
pub struct DbExecutor(pub SqliteConnectionPool);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

/// Message for getting verses.
pub struct VersesMessage {
    pub reference: Reference,
    pub format: VerseFormat,
}

/// Result returned from getting verses
type VersesResult = Result<(Book, Vec<Verse>), Error>;

impl Message for VersesMessage {
    type Result = VersesResult;
}

impl Handler<VersesMessage> for DbExecutor {
    type Result = VersesResult;

    fn handle(&mut self, msg: VersesMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::verses(&msg.reference, &msg.format, conn).map_err(map_db_err)
    }
}

/// Message for getting a book.
pub struct BookMessage {
    pub name: String,
}

/// Result returned from getting a book.
type BookResult = Result<(Book, Vec<i32>), Error>;

impl Message for BookMessage {
    type Result = BookResult;
}

impl Handler<BookMessage> for DbExecutor {
    type Result = BookResult;

    fn handle(&mut self, msg: BookMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::book(&msg.name, conn).map_err(map_db_err)
    }
}

/// Message for getting all books.
pub struct AllBooksMessage;

/// Result returned from getting all books.
type AllBooksResult = Result<Vec<Book>, Error>;

impl Message for AllBooksMessage {
    type Result = AllBooksResult;
}

impl Handler<AllBooksMessage> for DbExecutor {
    type Result = AllBooksResult;

    fn handle(&mut self, _: AllBooksMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::all_books(conn).map_err(map_db_err)
    }
}

/// Message for getting search results
pub struct SearchMessage {
    pub query: String,
}

/// Result returned from getting verses
type SearchResult = Result<Vec<(VerseFTS, Book)>, Error>;

impl Message for SearchMessage {
    type Result = SearchResult;
}

impl Handler<SearchMessage> for DbExecutor {
    type Result = SearchResult;

    fn handle(&mut self, msg: SearchMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::search(&msg.query, conn).map_err(map_db_err)
    }
}
