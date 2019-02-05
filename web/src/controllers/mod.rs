#![macro_use]

use std::ops::Range;

use actix_web::HttpRequest;
use serde_derive::Serialize;
use serde_json;
use url::Url;

use db::models::{Book, Reference, Verse, VerseFTS};

use crate::error::Error;
use crate::json_ld::{
    AboutJsonLd, AllBooksJsonLd, BookJsonLd, BreadcrumbListJsonLd, JsonLd, ListItemJsonLd,
    ReferenceJsonLd,
};

/// Name used in the HTML title generator
pub const NAME: &str = "Bible.rs";

/// Array of books indexed by their order in the Bible, including the
/// total number of chapters in that book.
const BOOKS: [(&str, i32); 68] = [
    ("_", 0), // Dummy in order to just use the book id (1-indexed)
    ("Genesis", 50),
    ("Exodus", 40),
    ("Leviticus", 27),
    ("Numbers", 36),
    ("Deuteronomy", 34),
    ("Joshua", 24),
    ("Judges", 21),
    ("Ruth", 4),
    ("1 Samuel", 31),
    ("2 Samuel", 24),
    ("1 Kings", 22),
    ("2 Kings", 25),
    ("1 Chronicles", 29),
    ("2 Chronicles", 36),
    ("Ezra", 10),
    ("Nehemiah", 13),
    ("Esther", 10),
    ("Job", 42),
    ("Psalms", 150),
    ("Proverbs", 31),
    ("Ecclesiastes", 12),
    ("Song of Solomon", 8),
    ("Isaiah", 66),
    ("Jeremiah", 52),
    ("Lamentations", 5),
    ("Ezekiel", 48),
    ("Daniel", 12),
    ("Hosea", 14),
    ("Joel", 3),
    ("Amos", 9),
    ("Obadiah", 1),
    ("Jonah", 4),
    ("Micah", 7),
    ("Nahum", 3),
    ("Habakkuk", 3),
    ("Zephaniah", 3),
    ("Haggai", 2),
    ("Zechariah", 14),
    ("Malachi", 4),
    ("Matthew", 28),
    ("Mark", 16),
    ("Luke", 24),
    ("John", 21),
    ("Acts", 28),
    ("Romans", 16),
    ("1 Corinthians", 16),
    ("2 Corinthians", 13),
    ("Galatians", 6),
    ("Ephesians", 6),
    ("Philippians", 4),
    ("Colossians", 4),
    ("1 Thessalonians", 5),
    ("2 Thessalonians", 3),
    ("1 Timothy", 6),
    ("2 Timothy", 4),
    ("Titus", 3),
    ("Philemon", 1),
    ("Hebrews", 13),
    ("James", 5),
    ("1 Peter", 5),
    ("2 Peter", 3),
    ("1 John", 5),
    ("2 John", 1),
    ("3 John", 1),
    ("Jude", 1),
    ("Revelation", 22),
    ("_", 0), // Dummy to avoid having to do a range check for the "next" book of Revelation
];

/// Error payload for a view (HTML or JSON)
#[derive(Clone, Serialize, Debug)]
struct ErrorPayload {
    message: String,
}

impl ErrorPayload {
    /// Creates a new error payload from a (db.Error.html)
    pub fn from_error(e: &Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

/// Generates a book URL for the given book.
fn book_url(b: &str, req: &HttpRequest) -> Link {
    Link::new(&req.url_for("book", &[b]).unwrap(), b.to_string())
}

/// Generates a chapter URL for the given book and chapter.
fn chapter_url(b: &str, c: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    Link::new(
        &req.url_for("reference", &[format!("{}/{}", b, chapter_string)])
            .unwrap(),
        format!("{} {}", b, chapter_string),
    )
}

/// Generates a verse URL from the given book, chapter, and verse.
fn verse_url(b: &str, c: i32, v: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    let verse_string = v.to_string();
    Link::new(
        &req.url_for(
            "reference",
            &[format!("{}/{}#v{}", b, chapter_string, verse_string)],
        )
        .unwrap(),
        format!("{} {}:{}", b, chapter_string, verse_string),
    )
}

/// Generates a URL for verses from the given book, chapter, and verse range.
fn verse_range_url(b: &str, c: i32, verses: &Range<i32>, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    let verses_string = if verses.start == verses.end {
        verses.start.to_string()
    } else {
        format!("{}-{}", verses.start, verses.end)
    };
    Link::new(
        &req.url_for(
            "reference",
            &[format!("{}/{}/{}", b, chapter_string, verses_string)],
        )
        .unwrap(),
        format!("{} {}:{}", b, chapter_string, verses_string),
    )
}

/// Link representing a URL and label
#[derive(Clone, Debug, Serialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

impl Link {
    fn new(url: &Url, label: String) -> Self {
        let url_string = if let Some(fragment) = url.fragment() {
            format!("{}#{}", url.path(), fragment)
        } else {
            url.path().to_string()
        };

        Self {
            label,
            url: url_string,
        }
    }
}

/// Links for the verses endpoint.
#[derive(Clone, Serialize, Debug)]
pub struct VersesLinks {
    pub books: Link,
    pub book: Link,
    pub chapter: Option<Link>,
    pub previous: Option<Link>,
    pub next: Option<Link>,
    pub current: Link,
}

impl VersesLinks {
    /// Creates a new structure of verses links.
    pub fn new(book: &Book, reference: &Reference, req: &HttpRequest) -> Self {
        let bible_root = Link::new(&req.url_for_static("bible").unwrap(), NAME.to_string());
        let book_link = Link::new(
            &req.url_for("book", &[&book.name]).unwrap(),
            book.name.to_string(),
        );
        let chapter_link = Some(chapter_url(&book.name, reference.chapter, req));
        let curr_link = match reference.verses {
            Some(ref vs) => verse_range_url(&book.name, reference.chapter, vs, req),
            None => chapter_url(&book.name, reference.chapter, req),
        };

        let (prev_link, next_link) = {
            let c = reference.chapter;
            let prev = BOOKS[book.id as usize - 1];
            let curr = BOOKS[book.id as usize];
            let next = BOOKS[book.id as usize + 1];

            // Genesis 1 (first chapter in the Bible)
            // Previous: nothing
            // Next: Genesis 2
            if book.id == 1 && c == 1 {
                (None, Some(chapter_url("Genesis", 2, req)))

            // Revelation 22 (last chapter in the Bible)
            // Previous: Revelation 21
            // Next: nothing
            } else if book.id == 66 && c == 22 {
                (Some(chapter_url("Revelation", 21, req)), None)

            // Books with one chapter
            // Previous: last chapter of the previous book
            // Next: first chapter of the last book
            } else if curr.1 == 1 {
                (
                    Some(chapter_url(prev.0, prev.1, req)),
                    Some(chapter_url(next.0, 1, req)),
                )
            // First chapter in a book
            // Previous: last chapter of the previous book
            // Next: next chapter in same book
            } else if c == 1 {
                (
                    Some(chapter_url(prev.0, prev.1, req)),
                    Some(chapter_url(&book.name, c + 1, req)),
                )
            // Last chapter in a book
            // Previous: last chapter in same book
            // Next: first chapter in next book
            } else if c == curr.1 {
                (
                    Some(chapter_url(&book.name, c - 1, req)),
                    Some(chapter_url(next.0, 1, req)),
                )
            // Everything else
            // Previous: last chapter in same book
            // Next: next chapter in same book
            } else {
                (
                    Some(chapter_url(&book.name, c - 1, req)),
                    Some(chapter_url(&book.name, c + 1, req)),
                )
            }
        };

        Self {
            books: bible_root,
            book: book_link,
            chapter: chapter_link,
            previous: prev_link,
            next: next_link,
            current: curr_link,
        }
    }
}

/// Represents a payload of verses (HTML or JSON).
#[derive(Clone, Serialize, Debug)]
pub struct VersesPayload {
    book: Book,
    links: VersesLinks,
    reference: Reference,
    reference_string: String,
    verses: Vec<Verse>,
}

impl VersesPayload {
    /// Creates a new payload for the verses page.
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

/// Links for the books endpoint.
#[derive(Clone, Serialize, Debug)]
pub struct BookLinks {
    pub books: Link,
    pub chapters: Vec<String>,
    pub previous: Option<Link>,
    pub next: Option<Link>,
    pub current: Link,
}

impl BookLinks {
    /// Creates a new structure of book links.
    pub fn new(book: &Book, chapters: &[i32], req: &HttpRequest) -> Self {
        Self {
            books: Link::new(&req.url_for_static("bible").unwrap(), NAME.to_string()),
            chapters: chapters
                .iter()
                .map(|c| chapter_url(&book.name, *c, req).url)
                .collect(),
            previous: if book.id != 1 {
                Some(book_url(BOOKS[book.id as usize - 1].0, req))
            } else {
                None
            },
            next: if book.id != 66 {
                Some(book_url(BOOKS[book.id as usize + 1].0, req))
            } else {
                None
            },
            current: book_url(&book.name, req),
        }
    }
}

/// Represents a payload for the books endpoint (HTML or JSON).
#[derive(Serialize)]
pub struct BookPayload {
    book: Book,
    chapters: Vec<i32>,
    links: BookLinks,
}

impl BookPayload {
    /// Creates a new book payload.
    fn new((book, chapters): (Book, Vec<i32>), req: &HttpRequest) -> Self {
        let links = BookLinks::new(&book, &chapters, req);
        Self {
            book,
            chapters,
            links,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct AllBooksLinks {
    pub books: Vec<Link>,
}

/// Payload for the "all books" endpoint (HTML or JSON).
#[derive(Serialize)]
pub struct AllBooksPayload {
    books: Vec<Book>,
    links: AllBooksLinks,
}

/// A search result.
#[derive(Serialize)]
pub struct SearchResult {
    link: Link,
    text: String,
}

/// Payload for the search endpoint (HTML or JSON).
#[derive(Serialize)]
pub struct SearchResultPayload {
    matches: Vec<SearchResult>,
}

impl SearchResultPayload {
    /// Creates an empty search result list.
    fn empty() -> Self {
        Self { matches: vec![] }
    }

    /// Creates a new search result payload from full text search verses.
    fn from_verses_fts(from_db: Vec<(VerseFTS, Book)>, req: &HttpRequest) -> Self {
        let matches = from_db.into_iter().map(|(v, b)| SearchResult {
            link: verse_url(&b.name, v.chapter, v.verse, req),
            text: v.words,
        });

        Self {
            matches: matches.collect(),
        }
    }

    /// Creates a new search result payload from standard verses.
    fn from_verses(from_db: (Book, Vec<Verse>), req: &HttpRequest) -> Self {
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

#[derive(Clone, Serialize, Debug)]
pub struct Meta {
    description: String,
    json_ld: Vec<JsonLd>,
    title: String,
    url: String,
}

macro_rules! title_format {
    () => {
        "Bible.rs | {}"
    };
}

macro_rules! url_format {
    () => {
        "https://bible.rs{}"
    };
}

impl Meta {
    fn for_about() -> Self {
        Self {
            description: "About Bible.rs".to_string(),
            json_ld: vec![JsonLd::About(Box::new(AboutJsonLd::new()))],
            title: format!(title_format!(), "About"),
            url: format!(url_format!(), "/about"),
        }
    }

    fn for_all_books(links: &AllBooksLinks) -> Self {
        Self {
            description: "Browse and search the King James version of the Bible using a lightning-fast and slick interface.".to_string(),
            json_ld: vec![JsonLd::AllBooks(AllBooksJsonLd::new(links))],
            title: format!(title_format!(), "King James Version"),
            url: format!(url_format!(), ""),
        }
    }

    fn for_book(book: &Book, links: &BookLinks) -> Self {
        Self {
            description: format!("The book of {}", book.name),
            json_ld: vec![
                JsonLd::Book(BookJsonLd::new(&book, links)),
                JsonLd::BreadcrumbList(BreadcrumbListJsonLd::new(vec![
                    ListItemJsonLd::new(&links.books, 1),
                    ListItemJsonLd::new(&links.current, 2),
                ])),
            ],
            title: format!(title_format!(), book.name),
            url: format!(url_format!(), links.current.url),
        }
    }

    fn for_error() -> Self {
        Self {
            description: "Error page".to_string(),
            json_ld: vec![],
            title: format!(title_format!(), "Error"),
            url: format!(url_format!(), ""),
        }
    }

    fn for_reference(reference: &Reference, verses: &[Verse], links: &VersesLinks) -> Self {
        let ref_string = reference.to_string();
        Self {
            description: match verses.first() {
                None => ref_string.to_owned(),
                Some(v) => format!("{}...", v.words),
            },
            json_ld: vec![
                JsonLd::Reference(ReferenceJsonLd::new(&reference, &links)),
                JsonLd::BreadcrumbList(BreadcrumbListJsonLd::new(vec![
                    ListItemJsonLd::new(&links.books, 1),
                    ListItemJsonLd::new(&links.book, 2),
                    ListItemJsonLd::new(&links.current, 3),
                ])),
            ],
            title: format!(title_format!(), ref_string),
            url: format!(url_format!(), links.current.url),
        }
    }

    fn for_search(query: &str, url: &str) -> Self {
        let results_string = format!("Results for '{}'", query);
        Self {
            description: results_string.to_owned(),
            json_ld: vec![],
            title: format!(title_format!(), results_string),
            url: format!(url_format!(), url),
        }
    }
}

pub mod api;
pub mod view;
