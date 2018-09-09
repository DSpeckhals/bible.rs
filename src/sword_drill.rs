use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::Sqlite;

use models::*;
use reference::Reference;
use ReceptusError;

pub fn verses<C>(reference: &Reference, conn: &C) -> Result<(Book, Vec<Verse>), ReceptusError>
where
    C: Connection<Backend = Sqlite>,
{
    use schema::book_abbreviations as ba;
    use schema::books as b;
    use schema::verses as v;

    // Get the book
    let (book, _) = b::table
        .inner_join(ba::table)
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

pub fn book<C>(book_name: &str, conn: &C) -> Result<(Book, Vec<i32>), ReceptusError>
where
    C: Connection<Backend = Sqlite>,
{
    use schema::book_abbreviations as ba;
    use schema::books as b;

    let (book, _) = b::table
        .inner_join(ba::table)
        .filter(ba::abbreviation.eq(book_name.to_lowercase()))
        .order_by(b::id)
        .first::<(Book, BookAbbreviation)>(conn)
        .map_err(|e| match e {
            Error::NotFound => ReceptusError::BookNotFound {
                book: book_name.to_owned(),
            },
            e => ReceptusError::DatabaseError { root_cause: e },
        })?;
    let chapters: Vec<i32> = (1..=book.chapter_count).collect();

    Ok((book, chapters))
}

pub fn all_books<C>(conn: &C) -> Result<Vec<Book>, ReceptusError>
where
    C: Connection<Backend = Sqlite>,
{
    use schema::books::dsl::*;

    books
        .order_by(id)
        .load(conn)
        .map_err(|e| ReceptusError::DatabaseError { root_cause: e })
}
