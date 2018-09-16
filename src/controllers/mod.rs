use actix_web::HttpRequest;
use serde_json;
use url::Url;
use url_serde;

use models::{Book, Verse, VerseFTS};
use reference::Reference;
use ReceptusError;

const NAME: &str = "Bible.rs";

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

#[derive(Serialize, Debug)]
struct ErrorPayload {
    message: String,
}

impl ErrorPayload {
    pub fn new(e: &ReceptusError) -> Self {
        Self {
            message: e.to_string(),
        }
    }

    pub fn json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

fn book_url(b: &str, req: &HttpRequest) -> Link {
    Link::new(req.url_for("book", &[b]).unwrap(), b.to_string())
}

fn chapter_url(b: &str, c: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    Link::new(
        req.url_for("reference", &[format!("{}/{}", b, chapter_string)])
            .unwrap(),
        format!("{} {}", b, chapter_string),
    )
}

fn verse_url(b: &str, c: i32, v: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    let verse_string = v.to_string();
    Link::new(
        req.url_for(
            "reference",
            &[format!("{}/{}#v{}", b, chapter_string, verse_string)],
        ).unwrap(),
        format!("{} {}:{}", b, chapter_string, verse_string),
    )
}

#[derive(Clone, Debug, Serialize)]
pub struct Link {
    label: String,

    #[serde(with = "url_serde")]
    url: Url,
}

impl Link {
    fn new(url: Url, label: String) -> Self {
        Self { label, url }
    }
}

#[derive(Serialize)]
pub struct VersesLinks {
    books: Link,
    book: Link,
    chapter: Option<Link>,
    previous: Option<Link>,
    next: Option<Link>,
    current: Link,
}

impl VersesLinks {
    pub fn new(book: &Book, reference: &Reference, req: &HttpRequest) -> Self {
        let bible_root = Link::new(req.url_for_static("bible").unwrap(), NAME.to_string());
        let book_link = Link::new(
            req.url_for("book", &[&book.name]).unwrap(),
            book.name.to_string(),
        );
        let (chapter_link, curr_link) = {
            (
                Some(chapter_url(&book.name, reference.chapter, req)),
                chapter_url(&book.name, reference.chapter, req),
            )
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

#[derive(Serialize)]
pub struct VersesPayload {
    book: Book,
    links: VersesLinks,
    reference: Reference,
    reference_string: String,
    verses: Vec<Verse>,
}

impl VersesPayload {
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

#[derive(Serialize)]
pub struct BookLinks {
    books: Link,
    chapters: Vec<String>,
    previous: Option<Link>,
    next: Option<Link>,
    current: Link,
}

impl BookLinks {
    pub fn new(book: &Book, chapters: &[i32], req: &HttpRequest) -> Self {
        Self {
            books: Link::new(req.url_for_static("bible").unwrap(), NAME.to_string()),
            chapters: chapters
                .iter()
                .map(|c| chapter_url(&book.name, *c, req).url.into_string())
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

#[derive(Serialize)]
pub struct BookPayload {
    book: Book,
    chapters: Vec<i32>,
    links: BookLinks,
}

impl BookPayload {
    fn new((book, chapters): (Book, Vec<i32>), req: &HttpRequest) -> Self {
        let links = BookLinks::new(&book, &chapters, req);
        Self {
            book,
            chapters,
            links,
        }
    }
}

#[derive(Serialize)]
pub struct AllBooksPayload {
    books: Vec<Book>,
}

#[derive(Serialize)]
pub struct SearchResult {
    link: Link,
    text: String,
}

#[derive(Serialize)]
pub struct SearchResultPayload {
    matches: Vec<SearchResult>,
}

impl SearchResultPayload {
    fn empty() -> Self {
        Self { matches: vec![] }
    }

    fn from_verses_fts(from_db: Vec<(VerseFTS, Book)>, req: &HttpRequest) -> Self {
        let matches = from_db.into_iter().map(|(v, b)| SearchResult {
            link: verse_url(&b.name, v.chapter, v.verse, req),
            text: v.words,
        });

        Self {
            matches: matches.collect(),
        }
    }

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

pub mod api;
pub mod view;
