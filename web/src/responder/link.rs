use std::ops::RangeInclusive;

use actix_web::error::UrlGenerationError;
use actix_web::HttpRequest;
use log::error;
use serde_derive::{Deserialize, Serialize};
use url::Url;

use db::models::{Book, Reference};

/// Name used in the HTML title generator
pub const NAME: &str = "Bible.rs";

fn invalid_url(e: UrlGenerationError) -> Url {
    error!("{:?}", e);
    Url::parse("https://bible.rs").unwrap()
}

/// Generates a book URL for the given book.
fn book_url(b: &str, req: &HttpRequest) -> Link {
    Link::new(
        &req.url_for("book", [b]).unwrap_or_else(invalid_url),
        b.to_string(),
    )
}

/// Generates a chapter URL for the given book and chapter.
fn chapter_url(b: &str, c: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    Link::new(
        &req.url_for("reference", [format!("{}/{}", b, chapter_string)])
            .unwrap_or_else(invalid_url),
        format!("{} {}", b, chapter_string),
    )
}

/// Generates a verse URL from the given book, chapter, and verse.
pub(super) fn verse_url(b: &str, c: i32, v: i32, req: &HttpRequest) -> Link {
    let chapter_string = c.to_string();
    let verse_string = v.to_string();
    let mut url = req
        .url_for("reference", [format!("{}/{}", b, chapter_string)])
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
            [format!("{}/{}/{}", b, chapter_string, verses_string)],
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
    pub(super) fn new(
        book: &Book,
        reference: &Reference,
        books: &[Book],
        req: &HttpRequest,
    ) -> Self {
        let bible_root = Link::new(&req.url_for_static("bible").unwrap(), NAME.to_string());
        let book_index = book.id as usize - 1;
        println!("{}", book_index);
        let book_link = Link::new(
            &req.url_for("book", [&book.name]).unwrap(),
            book.name.to_string(),
        );
        let chapter_link = Some(chapter_url(&book.name, reference.chapter, req));
        let current_link = match reference.verses {
            Some(ref vs) => verse_range_url(&book.name, reference.chapter, vs, req),
            None => chapter_url(&book.name, reference.chapter, req),
        };

        let (prev_link, next_link) = {
            let c = reference.chapter;
            let previous = if book_index == 0 {
                None
            } else {
                books.get(book_index - 1)
            };
            let next = books.get(book_index + 1);

            // First chapter in the Bible
            // Previous: nothing
            // Next: Genesis 2
            if previous.is_none() && c == 1 {
                (None, Some(chapter_url(&book.name, c + 1, req)))

            // Last chapter in the Bible
            // Previous: Revelation 21
            // Next: nothing
            } else if next.is_none() && c == book.chapter_count {
                (
                    Some(chapter_url(&book.name, book.chapter_count - 1, req)),
                    None,
                )

            // Books with one chapter
            // Previous: last chapter of the previous book
            // Next: first chapter of the last book
            } else if book.chapter_count == 1 {
                (
                    Some(chapter_url(
                        &previous.unwrap().name,
                        previous.unwrap().chapter_count,
                        req,
                    )),
                    Some(chapter_url(&next.unwrap().name, 1, req)),
                )
            // First chapter in a book
            // Previous: last chapter of the previous book
            // Next: next chapter in same book
            } else if c == 1 {
                (
                    Some(chapter_url(
                        &previous.unwrap().name,
                        previous.unwrap().chapter_count,
                        req,
                    )),
                    Some(chapter_url(&book.name, c + 1, req)),
                )
            // Last chapter in a book
            // Previous: last chapter in same book
            // Next: first chapter in next book
            } else if c == book.chapter_count {
                (
                    Some(chapter_url(&book.name, c - 1, req)),
                    Some(chapter_url(&next.unwrap().name, 1, req)),
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
            current: current_link,
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
    pub(super) fn new(book: &Book, chapters: &[i32], books: &[Book], req: &HttpRequest) -> Self {
        let book_index = book.id as usize - 1;
        Self {
            books: Link::new(&req.url_for_static("bible").unwrap(), NAME.to_string()),
            chapters: chapters
                .iter()
                .map(|c| chapter_url(&book.name, *c, req).url)
                .collect(),
            previous: if book.id != 1 {
                Some(book_url(&books[book_index - 1].name, req))
            } else {
                None
            },
            next: if book.id != books.len() as i32 {
                Some(book_url(&books[book_index + 1].name, req))
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

    #[actix_web::test]
    async fn urls_for_books() {
        with_service(|req| {
            // Genesis
            let book = BOOKS[0].clone();
            let links = BookLinks::new(
                &book,
                (1..=book.chapter_count).collect::<Vec<i32>>().as_slice(),
                &BOOKS,
                &req,
            );
            assert_eq!(links.current.url, "/Genesis");
            assert!(links.previous.is_none());
            assert_eq!(links.next.unwrap().url, "/Exodus");

            // Revelation
            let book = BOOKS[65].clone();
            let links = BookLinks::new(
                &book,
                (1..=book.chapter_count).collect::<Vec<i32>>().as_slice(),
                &BOOKS,
                &req,
            );
            assert_eq!(links.current.url, "/Revelation");
            assert_eq!(links.previous.unwrap().url, "/Jude");
            assert!(links.next.is_none());

            // Typical Book
            let book = BOOKS[18].clone();
            let links = BookLinks::new(
                &book,
                (1..=book.chapter_count).collect::<Vec<i32>>().as_slice(),
                &BOOKS,
                &req,
            );
            assert_eq!(links.current.url, "/Psalms");
            assert_eq!(links.previous.unwrap().url, "/Job");
            assert_eq!(links.next.unwrap().url, "/Proverbs");
        })
        .await;
    }

    #[actix_web::test]
    async fn urls_for_verses() {
        with_service(|req| {
            // Genesis 1
            let book = BOOKS[0].clone();
            let reference: Reference = "Genesis 1".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &BOOKS, &req);
            assert_eq!(links.book.url, "/Genesis");
            assert_eq!(links.chapter.unwrap().url, "/Genesis/1");
            assert_eq!(links.current.url, "/Genesis/1");
            assert!(links.previous.is_none());
            assert_eq!(links.next.unwrap().url, "/Genesis/2");

            // Revelation 22
            let book = BOOKS[65].clone();
            let reference: Reference = "Revelation 22".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &BOOKS, &req);
            assert_eq!(links.current.url, "/Revelation/22");
            assert_eq!(links.previous.unwrap().url, "/Revelation/21");
            assert!(links.next.is_none());

            // First chapter
            let book = BOOKS[18].clone();
            let reference: Reference = "Psalms 1".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &BOOKS, &req);
            assert_eq!(links.current.url, "/Psalms/1");
            assert_eq!(links.previous.unwrap().url, "/Job/42");

            // Last chapter
            let reference: Reference = "Psalms 150".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &BOOKS, &req);
            assert_eq!(links.current.url, "/Psalms/150");
            assert_eq!(links.next.unwrap().url, "/Proverbs/1");

            // Typical verses
            let reference: Reference = "Psalms 119".parse().unwrap();
            let links = VersesLinks::new(&book, &reference, &BOOKS, &req);
            assert_eq!(links.current.url, "/Psalms/119");
            assert_eq!(links.previous.unwrap().url, "/Psalms/118");
            assert_eq!(links.next.unwrap().url, "/Psalms/120");
        })
        .await;
    }
}
