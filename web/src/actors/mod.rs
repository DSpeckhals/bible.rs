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
