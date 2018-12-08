use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::{Integer, Text};
use diesel::sqlite::Sqlite;
use regex::Regex;

use crate::models::*;
use crate::{DbError, VerseFormat};

sql_function!(
    /// Represents the [`highlight` function](https://sqlite.org/fts5.html#the_highlight_function)
    /// from SQLite's FTS5 extension.
    fn highlight(table_name: Text, column_index: Integer, prefix: Text, suffix: Text) -> Text;
);

/// Looks up Bible verses for the given reference.
pub fn verses<C>(
    reference: &Reference,
    format: &VerseFormat,
    conn: &C,
) -> Result<(Book, Vec<Verse>), DbError>
where
    C: Connection<Backend = Sqlite>,
{
    use crate::schema::verses as plain_text;
    use crate::schema::verses_html as html;

    let (book, _) = book(&reference.book.to_lowercase(), conn)?;

    match format {
        VerseFormat::PlainText => {
            let mut query = plain_text::table
                .filter(plain_text::book.eq(book.id))
                .filter(plain_text::chapter.eq(reference.chapter))
                .order_by((plain_text::chapter.asc(), plain_text::verse.asc()))
                .into_boxed();

            if let Some(ref verses) = reference.verses {
                query = query.filter(plain_text::verse.between(verses.start, verses.end));
            }
            query.load(conn)
        }
        VerseFormat::HTML => {
            let mut query = html::table
                .filter(html::book.eq(book.id))
                .filter(html::chapter.eq(reference.chapter))
                .order_by((html::chapter.asc(), html::verse.asc()))
                .into_boxed();

            if let Some(ref verses) = reference.verses {
                query = query.filter(html::verse.between(verses.start, verses.end));
            }
            query.load(conn)
        }
    }
    .and_then(|verses| Ok((book, verses)))
    .map_err(|e| DbError::Other { cause: e })
}

/// Looks up the Bible book with the given book name.
///
/// The inputted name argument can be either the cannonical book name
/// or an acceptable abbreviation defined in the database's
/// abbreviation table. The book is looked up in a case-insensitive
/// manner.
///
/// If found, returns the resulting book and the list of its chapters.
pub fn book<C>(book_name: &str, conn: &C) -> Result<(Book, Vec<i32>), DbError>
where
    C: Connection<Backend = Sqlite>,
{
    use crate::schema::book_abbreviations as ba;
    use crate::schema::books as b;

    let (book, _) = b::table
        .inner_join(ba::table)
        .filter(ba::abbreviation.eq(book_name.to_lowercase()))
        .first::<(Book, BookAbbreviation)>(conn)
        .map_err(|e| match e {
            Error::NotFound => DbError::BookNotFound {
                book: book_name.to_owned(),
            },
            e => DbError::Other { cause: e },
        })?;
    let chapters: Vec<i32> = (1..=book.chapter_count).collect();

    Ok((book, chapters))
}

/// Gets all books in the Bible.
pub fn all_books<C>(conn: &C) -> Result<Vec<Book>, DbError>
where
    C: Connection<Backend = Sqlite>,
{
    use crate::schema::books::dsl::*;

    books
        .order_by(id)
        .load(conn)
        .map_err(|e| DbError::Other { cause: e })
}

/// Max number of search results returned from the database.
const SEARCH_RESULT_LIMIT: i64 = 15;

/// Searches the database using the SQLite 3 full text search extension.
///
/// The inputted query string can be of two different formats:
///
/// - `test foo`: match each word as its own token, and use that
/// to search.
/// - `"test foo"`: match the entire phrase. For the King James
/// version of the Bible, this is safe because there are no literal
/// quotation marks. This cannot be assumed safe in other translations.
///
/// All characters other than alpha and quotations are stripped out.
pub fn search<C>(query: &str, conn: &C) -> Result<Vec<(VerseFTS, Book)>, DbError>
where
    C: Connection<Backend = Sqlite>,
{
    use crate::schema::books;
    use crate::schema::verses_fts;

    lazy_static! {
        static ref ALPHA_NUM: Regex = Regex::new(r"[^a-zA-Z ]+").unwrap();
    }

    let had_quote = query.contains('"');

    // Replace all characters that aren't alpha or space
    let mut query = ALPHA_NUM.replace_all(query, "").to_string();

    // Don't even try to run the query if there are no characters
    if query.is_empty() {
        return Ok(vec![]);
    }

    // Add back quotes safely if it had a quote before, and was removed
    // This makes FTS5 query the string as a phrase.
    query = if had_quote {
        format!("\"{}\"", query)
    } else {
        query
    };

    verses_fts::table
        .inner_join(books::table.on(books::id.eq(verses_fts::book)))
        .select((
            (
                verses_fts::book,
                verses_fts::chapter,
                verses_fts::verse,
                highlight(verses_fts::text, 3, "<em>", "</em>"),
                verses_fts::rank,
            ),
            (
                books::id,
                books::name,
                books::chapter_count,
                books::testament,
            ),
        ))
        .filter(verses_fts::text.eq(format!("{}*", query)))
        .order_by(verses_fts::rank)
        .limit(SEARCH_RESULT_LIMIT)
        .load::<(VerseFTS, Book)>(conn)
        .map_err(|e| DbError::Other { cause: e })
}
