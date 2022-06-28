use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::{Integer, Text};
use lazy_static::lazy_static;
use regex::Regex;

use crate::models::*;
use crate::{DbError, VerseFormat};

/// Max number of search results returned from the database.
const SEARCH_RESULT_LIMIT: i64 = 15;

sql_function!(
    fn highlight(table_name: Text, column_index: Integer, prefix: Text, suffix: Text) -> Text
);

/// Trait implemented by types that can query for and return types of Bible structures.
pub trait SwordDrillable {
    /// Looks up Bible verses for the given reference.
    fn verses(
        reference: &Reference,
        format: VerseFormat,
        conn: &mut SqliteConnection,
    ) -> Result<(Book, Vec<Verse>), DbError>;

    /// Looks up the Bible book with the given book name.
    ///
    /// The inputted name argument can be either the cannonical book name
    /// or an acceptable abbreviation defined in the database's
    /// abbreviation table. The book is looked up in a case-insensitive
    /// manner.
    ///
    /// If found, returns the resulting book and the list of its chapters.
    fn book(book_name: &str, conn: &mut SqliteConnection) -> Result<(Book, Vec<i32>), DbError>;

    /// Gets all books in the Bible.
    fn all_books(conn: &mut SqliteConnection) -> Result<Vec<Book>, DbError>;

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
    fn search(query: &str, conn: &mut SqliteConnection) -> Result<Vec<(VerseFTS, Book)>, DbError>;
}

/// Main implementation for the [SwordDrillable](crate::sword_drill::SwordDrillable) trait.
pub struct SwordDrill;

impl SwordDrillable for SwordDrill {
    fn verses(
        reference: &Reference,
        format: VerseFormat,
        conn: &mut SqliteConnection,
    ) -> Result<(Book, Vec<Verse>), DbError> {
        use crate::schema::verses as plain_text;
        use crate::schema::verses_html as html;

        let (book, _) = Self::book(&reference.book.to_lowercase(), conn)?;

        match format {
            VerseFormat::PlainText => {
                let mut query = plain_text::table
                    .filter(plain_text::book.eq(book.id))
                    .filter(plain_text::chapter.eq(reference.chapter))
                    .order_by((plain_text::chapter.asc(), plain_text::verse.asc()))
                    .into_boxed();

                if let Some(ref verses) = reference.verses {
                    query = query.filter(plain_text::verse.between(verses.start(), verses.end()));
                }
                query.load(conn)
            }
            VerseFormat::Html => {
                let mut query = html::table
                    .filter(html::book.eq(book.id))
                    .filter(html::chapter.eq(reference.chapter))
                    .order_by((html::chapter.asc(), html::verse.asc()))
                    .into_boxed();

                if let Some(ref verses) = reference.verses {
                    query = query.filter(html::verse.between(verses.start(), verses.end()));
                }
                query.load(conn)
            }
        }
        .map(|verses| (book, verses))
        .map_err(|e| DbError::Other {
            cause: e.to_string(),
        })
    }

    fn book(book_name: &str, conn: &mut SqliteConnection) -> Result<(Book, Vec<i32>), DbError> {
        use crate::schema::book_abbreviations as ba;
        use crate::schema::books as b;

        let (book, _): (Book, BookAbbreviation) = b::table
            .inner_join(ba::table)
            .filter(ba::abbreviation.eq(book_name.to_lowercase()))
            .first::<(Book, BookAbbreviation)>(conn)
            .map_err(|e| match e {
                Error::NotFound => DbError::BookNotFound {
                    book: book_name.to_owned(),
                },
                e => DbError::Other {
                    cause: e.to_string(),
                },
            })?;
        let chapters: Vec<i32> = (1..=book.chapter_count).collect();

        Ok((book, chapters))
    }

    fn all_books(conn: &mut SqliteConnection) -> Result<Vec<Book>, DbError> {
        use crate::schema::books::dsl::*;

        books.order_by(id).load(conn).map_err(|e| DbError::Other {
            cause: e.to_string(),
        })
    }

    fn search(query: &str, conn: &mut SqliteConnection) -> Result<Vec<(VerseFTS, Book)>, DbError> {
        use crate::schema::books;
        use crate::schema::verses_fts;

        lazy_static! {
            static ref ALPHA_NUM: Regex = Regex::new(r"[^a-zA-Z ]+").unwrap();
        }

        let had_quote = query.contains('"');

        // Replace all characters that aren't alpha or space
        let mut query = ALPHA_NUM.replace_all(query, "").to_string();

        // Don't even try to run the query if there are no characters
        if query.trim().is_empty() {
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
            .map_err(|e| DbError::Other {
                cause: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use diesel_migrations::{FileBasedMigrations, MigrationHarness};

    use super::*;
    use crate::establish_connection;

    #[test]
    fn all() {
        let mut conn = {
            let mut conn = establish_connection(":memory:");
            let source =
                FileBasedMigrations::find_migrations_directory_in_path(Path::new("./migrations"))
                    .unwrap();
            conn.run_pending_migrations(source).unwrap();
            conn
        };

        conn.test_transaction::<_, DbError, _>(|c| {
            // Verses
            {
                let result = SwordDrill::verses(
                &"Psalms 119:105".parse().unwrap(),
                VerseFormat::PlainText,
                c,
            )?;

            assert_eq!(result.0.name, "Psalms");
            assert_eq!(
                result.1[0].words,
                "NUN. Thy word is a lamp unto my feet, and a light unto my path."
            );
            }

            // Book
            {
                let result = SwordDrill::book("psa", c)?;

            assert_eq!(result.0.name, "Psalms");
            assert_eq!(result.1.len(), 150);
            }

            // All books
            {
                let result = SwordDrill::all_books(c)?;

            assert_eq!(result.len(), 66);
            assert_eq!(result[64].name, "Jude");
            }

            // Search - Fuzzy words
            {
                let result = SwordDrill::search("fire hammer rock", c)?;

                assert_eq!(result.len(), 1);
                assert_eq!(result[0].0.book, 24);
                assert_eq!(result[0].0.chapter, 23);
                assert_eq!(result[0].0.verse, 29);
                assert_eq!(
                    result[0].0.words,
                    "Is not my word like as a <em>fire</em>? saith the LORD; and like a <em>hammer</em> that breaketh the <em>rock</em> in pieces?",
                );
                assert_eq!(result[0].1.name, "Jeremiah");
            }

            // Search - Leading number followed by a space returns an empty result
            {
                let result = SwordDrill::search("1 ", c)?;
                assert_eq!(result.len(), 0);
            }

            // Search - Phrase
            {
                let result = SwordDrill::search("\"like as a fire\"", c)?;

                assert_eq!(result.len(), 1);
                assert_eq!(result[0].0.book, 24);
                assert_eq!(result[0].0.chapter, 23);
                assert_eq!(result[0].0.verse, 29);
                assert_eq!(
                    result[0].0.words,
                    "Is not my word <em>like as a fire</em>? saith the LORD; and like a hammer that breaketh the rock in pieces?",
                );
                assert_eq!(result[0].1.name, "Jeremiah");
            }
            Ok(())
        });
    }
}
