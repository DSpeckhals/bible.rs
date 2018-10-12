use actix::prelude::*;

use db::models::*;
use db::sword_drill;
use db::SqliteConnectionPool;
use db::{DbError, VerseFormat};

pub struct DbExecutor(pub SqliteConnectionPool);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct VersesMessage {
    pub reference: Reference,
    pub format: VerseFormat,
}

impl Message for VersesMessage {
    type Result = Result<(Book, Vec<Verse>), DbError>;
}

impl Handler<VersesMessage> for DbExecutor {
    type Result = Result<(Book, Vec<Verse>), DbError>;

    fn handle(&mut self, msg: VersesMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::verses(&msg.reference, &msg.format, conn)
    }
}

pub struct BookMessage {
    pub name: String,
}

impl Message for BookMessage {
    type Result = Result<(Book, Vec<i32>), DbError>;
}

impl Handler<BookMessage> for DbExecutor {
    type Result = Result<(Book, Vec<i32>), DbError>;

    fn handle(&mut self, msg: BookMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::book(&msg.name, conn)
    }
}

pub struct AllBooksMessage;

impl Message for AllBooksMessage {
    type Result = Result<Vec<Book>, DbError>;
}

impl Handler<AllBooksMessage> for DbExecutor {
    type Result = Result<Vec<Book>, DbError>;

    fn handle(&mut self, _: AllBooksMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::all_books(conn)
    }
}

pub struct SearchMessage {
    pub query: String,
}

impl Message for SearchMessage {
    type Result = Result<Vec<(VerseFTS, Book)>, DbError>;
}

impl Handler<SearchMessage> for DbExecutor {
    type Result = Result<Vec<(VerseFTS, Book)>, DbError>;

    fn handle(&mut self, msg: SearchMessage, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        sword_drill::search(&msg.query, conn)
    }
}
