use actix_web::HttpRequest;
use handlebars::Handlebars;
use log::error;
use serde;
use serde_derive::{Deserialize, Serialize};

use db::models::{Book, Reference, Verse, VerseFTS};

use crate::error::Error;
use crate::responder::link::{verse_url, AllBooksLinks, BookLinks, Link, VersesLinks};
use crate::responder::meta::Meta;

/// Represents empty data.
///
/// This is used to render Handlebars templates that don't
/// need any context to render (e.g. the About page).
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EmptyData;

/// Error data for a view (HTML or JSON)
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ErrorData {
    message: String,
}

impl ErrorData {
    /// Creates new error data from a (db.Error.html)
    pub fn from_error(e: &Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

/// Represents data of verses (HTML or JSON).
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct VersesData {
    pub book: Book,
    pub links: VersesLinks,
    pub reference: Reference,
    pub reference_string: String,
    pub verses: Vec<Verse>,
}

impl VersesData {
    /// Creates new data for the verses page.
    pub fn new(
        (book, verses): (Book, Vec<Verse>),
        mut reference: Reference,
        req: &HttpRequest,
    ) -> Self {
        reference.book = book.name.to_owned();
        reference.verses = if let Some(vs) = reference.verses {
            match verses.len() {
                0 => None,
                n => Some(vs.start..verses[n - 1].verse),
            }
        } else {
            None
        };
        let reference_string = reference.to_string();
        let links = VersesLinks::new(&book, &reference, &req);

        Self {
            book,
            links,
            reference,
            reference_string,
            verses,
        }
    }
}

/// Represents data for the books endpoint (HTML or JSON).
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BookData {
    pub book: Book,
    pub chapters: Vec<i32>,
    pub links: BookLinks,
}

impl BookData {
    /// Creates new book data.
    pub fn new((book, chapters): (Book, Vec<i32>), req: &HttpRequest) -> Self {
        let links = BookLinks::new(&book, &chapters, req);
        Self {
            book,
            chapters,
            links,
        }
    }
}

/// Data for the "all books" endpoint (HTML or JSON).
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AllBooksData {
    books: Vec<Book>,
    pub links: AllBooksLinks,
}

impl AllBooksData {
    pub fn new(books: Vec<Book>, req: &HttpRequest) -> Self {
        let links = AllBooksLinks::new(books.to_owned(), &req);
        Self { books, links }
    }
}

/// A search result.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SearchResult {
    link: Link,
    pub text: String,
}

/// Data for the search endpoint (HTML or JSON).
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SearchResultData {
    pub matches: Vec<SearchResult>,
}

impl SearchResultData {
    /// Creates an empty search result list.
    pub fn empty() -> Self {
        Self { matches: vec![] }
    }

    /// Creates new search result data from full text search verses.
    pub fn from_verses_fts(from_db: Vec<(VerseFTS, Book)>, req: &HttpRequest) -> Self {
        let matches = from_db.into_iter().map(|(v, b)| SearchResult {
            link: verse_url(&b.name, v.chapter, v.verse, req),
            text: v.words,
        });

        Self {
            matches: matches.collect(),
        }
    }

    /// Creates new search result data from standard verses.
    pub fn from_verses(from_db: (Book, Vec<Verse>), req: &HttpRequest) -> Self {
        let name = from_db.0.name;
        let matches = from_db.1.into_iter().map(|v| SearchResult {
            link: verse_url(&name, v.chapter, v.verse, req),
            text: v.words,
        });

        Self {
            matches: matches.collect(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TemplateData<T: serde::Serialize> {
    data: T,
    meta: Meta,
}

impl<T: serde::Serialize> TemplateData<T> {
    /// Create new HTML template Data.
    pub fn new(data: T, meta: Meta) -> Self {
        Self { data, meta }
    }

    /// Convert the template data to HTML
    pub fn to_html(&self, tpl_name: &str, renderer: &Handlebars) -> Result<String, Error> {
        renderer.render(tpl_name, &self).map_err(|e| {
            error!("{}", e);
            Error::Template
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;

    use db::models::*;

    use crate::responder::link::BOOKS;
    use crate::responder::meta::Meta;
    use crate::test::*;

    #[test]
    fn verses_data() {
        with_service(|req| {
            let prov = BOOKS[20];
            let book = Book {
                id: 20,
                name: prov.0.to_string(),
                testament: Testament::Old,
                chapter_count: prov.1,
            };
            let verses = vec![Verse {
                book: 20,
                chapter: 3,
                id: 555,
                verse: 5,
                words: "Trust in the LORD with all thine heart; and lean not unto thine own understanding.".to_string(),

            }];
            let reference: Reference = "Proverbs 3:5".parse().unwrap();
            let data = VersesData::new((book, verses), reference, &req);

            assert_eq!(data.reference_string, "Proverbs 3:5");
            assert_eq!(data.reference.book, "Proverbs");
            assert_eq!(data.verses.len(), 1);
        });
    }

    #[test]
    fn book_data() {
        with_service(|req| {
            let prov = BOOKS[20];
            let book = Book {
                id: 20,
                name: prov.0.to_string(),
                testament: Testament::Old,
                chapter_count: prov.1,
            };
            let chapters = (1..=book.chapter_count).collect::<Vec<i32>>();
            let data = BookData::new((book, chapters), &req);

            assert_eq!(data.book.name, "Proverbs");
            assert_eq!(data.chapters.len(), 31);
        });
    }

    #[test]
    fn search_result_data() {
        with_service(|req| {
            let prov = BOOKS[20];
            let book = Book {
                id: 20,
                name: prov.0.to_string(),
                testament: Testament::Old,
                chapter_count: prov.1,
            };
            let book_2 = book.clone();
            let verses = vec![Verse {
                book: 20,
                chapter: 3,
                id: 555,
                verse: 5,
                words: "Trust in the LORD with all thine heart; and lean not unto thine own understanding.".to_string(),

            }];
            let data = SearchResultData::from_verses((book, verses), &req);
            assert_eq!(data.matches.len(), 1);

            let results = vec![(VerseFTS {
                book: 20,
                chapter: 3,
                rank: 0.98,
                verse: 5,
                words: "Trust in the LORD with all thine heart; and lean not unto thine own understanding.".to_string(),
            }, book_2)];
            let data = SearchResultData::from_verses_fts(results, &req);
            assert_eq!(data.matches.len(), 1);
        });
    }

    #[test]
    fn template_data() {
        let mut tpl = Handlebars::new();
        tpl.register_template_string("test", "<html></html")
            .unwrap();
        let data = TemplateData::new(EmptyData {}, Meta::for_about());
        let html = data.to_html("test", &tpl).unwrap();
        assert!(html.starts_with("<html>"));
    }
}
