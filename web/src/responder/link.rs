use std::ops::RangeInclusive;

use actix_web::error::UrlGenerationError;
use actix_web::HttpRequest;
use log::error;
use serde_derive::{Deserialize, Serialize};
use url::Url;

use db::models::{Book, Reference};

/// Name used in the HTML title generator
pub const NAME: &str = "Bible.rs";

/// Array of books indexed by their order in the Bible, including the
/// total number of chapters in that book.
pub const BOOKS: [(&str, i32); 68] = [
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

fn invalid_url(e: UrlGenerationError) -> Url {
    error!("{:?}", e);
    Url::parse("https://bible.rs").unwrap()
}

/// Generates a book URL for the given book.
fn book_url(b: &str, req: &HttpRequest) -> Link {
    Link::new(
        &req.url_for("book", &[b]).unwrap_or_else(invalid_url),
        b.to_string(),
    )
}

/// Generates a chapter URL for the given book and chapter.
fn chapter_url(b: &str, c: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    Link::new(
        &req.url_for("reference", &[format!("{}/{}", b, chapter_string)])
            .unwrap_or_else(invalid_url),
        format!("{} {}", b, chapter_string),
    )
}

/// Generates a verse URL from the given book, chapter, and verse.
pub(super) fn verse_url(b: &str, c: i32, v: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    let verse_string = v.to_string();
    let mut url = req
        .url_for("reference", &[format!("{}/{}", b, chapter_string)])
        .unwrap_or_else(invalid_url);
    url.set_fragment(Some(&format!("v{}", verse_string)));
    Link::new(&url, format!("{} {}:{}", b, chapter_string, verse_string))
}

/// Generates a URL for verses from the given book, chapter, and verse range.
fn verse_range_url(b: &str, c: i32, verses: &RangeInclusive<i32>, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    let verses_string = if verses.start() == verses.end() {
        verses.start().to_string()
    } else {
        format!("{}-{}", verses.start(), verses.end())
    };
    Link::new(
        &req.url_for(
            "reference",
            &[format!("{}/{}/{}", b, chapter_string, verses_string)],
        )
        .unwrap_or_else(invalid_url),
        format!("{} {}:{}", b, chapter_string, verses_string),
    )
}

/// Link representing a URL and label
#[derive(Clone, Deserialize, Serialize, Debug)]
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
#[derive(Clone, Deserialize, Serialize, Debug)]
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
    pub(super) fn new(book: &Book, reference: &Reference, req: &HttpRequest) -> Self {
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

/// Links for the books endpoint.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BookLinks {
    pub books: Link,
    pub chapters: Vec<String>,
    pub previous: Option<Link>,
    pub next: Option<Link>,
    pub current: Link,
}

impl BookLinks {
    /// Creates a new structure of book links.
    pub(super) fn new(book: &Book, chapters: &[i32], req: &HttpRequest) -> Self {
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

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AllBooksLinks {
    pub books: Vec<Link>,
}

impl AllBooksLinks {
    pub(super) fn new(books: Vec<Book>, req: &HttpRequest) -> Self {
        Self {
            books: books.iter().map(|b| book_url(&b.name, req)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;
    use db::models::*;

    #[test]
    fn urls_for_books() {
        with_service(|req| {
            // Genesis
            let gen = BOOKS[1];
            let book = Book {
                id: 1,
                name: gen.0.to_string(),
                testament: Testament::Old,
                chapter_count: gen.1,
            };
            let links = BookLinks::new(
                &book,
                (1..=book.chapter_count).collect::<Vec<i32>>().as_slice(),
                &req,
            );
            assert_eq!(links.current.url, "/Genesis");
            assert!(links.previous.is_none());
            assert_eq!(links.next.unwrap().url, "/Exodus");

            // Revelation
            let rev = BOOKS[66];
            let book = Book {
                id: 66,
                name: rev.0.to_string(),
                testament: Testament::New,
                chapter_count: rev.1,
            };
            let links = BookLinks::new(
                &book,
                (1..=book.chapter_count).collect::<Vec<i32>>().as_slice(),
                &req,
            );
            assert_eq!(links.current.url, "/Revelation");
            assert_eq!(links.previous.unwrap().url, "/Jude");
            assert!(links.next.is_none());

            // Typical Book
            let psa = BOOKS[19];
            let book = Book {
                id: 19,
                name: psa.0.to_string(),
                testament: Testament::Old,
                chapter_count: psa.1,
            };
            let links = BookLinks::new(
                &book,
                (1..=book.chapter_count).collect::<Vec<i32>>().as_slice(),
                &req,
            );
            assert_eq!(links.current.url, "/Psalms");
            assert_eq!(links.previous.unwrap().url, "/Job");
            assert_eq!(links.next.unwrap().url, "/Proverbs");
        });
    }

    #[test]
    fn urls_for_verses() {
        with_service(|req| {
            // Genesis 1
            let gen = BOOKS[1];
            let book = Book {
                id: 1,
                name: gen.0.to_string(),
                testament: Testament::Old,
                chapter_count: gen.1,
            };
            let reference: Reference = "Genesis 1".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &req);
            assert_eq!(links.book.url, "/Genesis");
            assert_eq!(links.chapter.unwrap().url, "/Genesis/1");
            assert_eq!(links.current.url, "/Genesis/1");
            assert!(links.previous.is_none());
            assert_eq!(links.next.unwrap().url, "/Genesis/2");

            // Revelation 22
            let rev = BOOKS[66];
            let book = Book {
                id: 66,
                name: rev.0.to_string(),
                testament: Testament::New,
                chapter_count: rev.1,
            };
            let reference: Reference = "Revelation 22".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &req);
            assert_eq!(links.current.url, "/Revelation/22");
            assert_eq!(links.previous.unwrap().url, "/Revelation/21");
            assert!(links.next.is_none());

            // First chapter
            let psa = BOOKS[19];
            let book = Book {
                id: 19,
                name: psa.0.to_string(),
                testament: Testament::Old,
                chapter_count: psa.1,
            };
            let reference: Reference = "Psalms 1".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &req);
            assert_eq!(links.current.url, "/Psalms/1");
            assert_eq!(links.previous.unwrap().url, "/Job/42");

            // Last chapter
            let reference: Reference = "Psalms 150".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &req);
            assert_eq!(links.current.url, "/Psalms/150");
            assert_eq!(links.next.unwrap().url, "/Proverbs/1");

            // Typical verses
            let reference: Reference = "Psalms 119".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &req);
            assert_eq!(links.current.url, "/Psalms/119");
            assert_eq!(links.previous.unwrap().url, "/Psalms/118");
            assert_eq!(links.next.unwrap().url, "/Psalms/120");
        });
    }
}
