use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::Sqlite;

use models::*;
use reference::Reference;
use ReceptusError;

pub fn drill<C: Connection<Backend = Sqlite>>(
    reference: &Reference,
    conn: &C,
) -> Result<(Book, Vec<Verse>), ReceptusError> {
    use schema::book_abbreviations as ba;
    use schema::books as b;
    use schema::verses as v;

    // Get the book
    let (book, _) = b::table
        .inner_join(ba::table.on(b::id.eq(ba::book_id)))
        .filter(ba::abbreviation.eq(reference.book.to_lowercase()))
        .first::<(Book, BookAbbreviation)>(conn)
        .map_err(|e| match e {
            Error::NotFound => ReceptusError::BookNotFound {
                book: reference.book.to_owned(),
            },
            e => ReceptusError::DatabaseError { root_cause: e },
        })?;

    let mut query = v::table.filter(v::book.eq(book.id)).into_boxed();

    if let Some(chapter) = reference.chapter {
        query = query.filter(v::chapter.eq(chapter));
    }

    if let Some(ref verses) = reference.verses {
        query = query.filter(v::verse.between(verses.start, verses.end));
    }

    query
        .order_by((v::chapter.asc(), v::verse.asc()))
        .load(conn)
        .and_then(|verses| Ok((book, verses)))
        .map_err(|e| ReceptusError::DatabaseError { root_cause: e })
}
