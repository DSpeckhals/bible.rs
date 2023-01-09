use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

use crate::DbError;

/// Model representing a Bible reference used to look up a
/// passage in the database.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Reference {
    pub book: String,
    pub chapter: i32,
    pub verses: Option<RangeInclusive<i32>>,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reference {
                book,
                chapter,
                verses: None,
            } => write!(f, "{} {}", book, chapter),
            Reference {
                book,
                chapter,
                verses: Some(verses),
            } => {
                if verses.start() == verses.end() {
                    write!(f, "{} {}:{}", book, chapter, verses.start())
                } else {
                    write!(
                        f,
                        "{} {}:{}-{}",
                        book,
                        chapter,
                        verses.start(),
                        verses.end()
                    )
                }
            }
        }
    }
}

const MAX_REFERENCE_SIZE: usize = 100;

enum State {
    Init,
    Book,
    Chapter,
    VerseFrom,
    VerseTo,
}

impl FromStr for Reference {
    type Err = DbError;

    fn from_str(s: &str) -> Result<Reference, Self::Err> {
        if s.len() > MAX_REFERENCE_SIZE || s.is_empty() {
            return Err(DbError::InvalidReference {
                reference: s[..MAX_REFERENCE_SIZE].to_string(),
            });
        }

        let mut state = State::Init;
        let mut book_part = String::new();
        let mut chapter_part = String::new();
        let mut verse_from_part = String::new();
        let mut verse_to_part = String::new();
        for c in s.chars() {
            match state {
                State::Init => {
                    book_part.push(c);
                    state = State::Book;
                }
                State::Book => {
                    if c.is_numeric() {
                        chapter_part.push(c);
                        state = State::Chapter;
                    } else if c.is_alphabetic() || c.is_whitespace() {
                        book_part.push(c);
                    }
                }
                State::Chapter => {
                    if c.is_numeric() {
                        chapter_part.push(c);
                    } else if c == ':' || c == '.' {
                        state = State::VerseFrom;
                    }
                }
                State::VerseFrom => {
                    if c == '-' {
                        state = State::VerseTo;
                    } else if c.is_numeric() {
                        verse_from_part.push(c);
                    }
                }
                State::VerseTo => {
                    if c.is_numeric() {
                        verse_to_part.push(c);
                    }
                }
            }
        }

        let r = Reference {
            book: book_part.trim().to_string(),
            chapter: chapter_part
                .parse()
                .map_err(|_| DbError::InvalidReference {
                    reference: s.to_string(),
                })?,
            verses: if verse_from_part.is_empty() {
                None
            } else {
                Some(RangeInclusive::new(
                    parse_num(verse_from_part.clone())?,
                    if verse_to_part.is_empty() {
                        parse_num(verse_from_part)?
                    } else {
                        parse_num(verse_to_part)?
                    },
                ))
            },
        };

        Ok(r)
    }
}

/// Parse a [Match](String.html) into an i32.
fn parse_num(s: String) -> Result<i32, DbError> {
    s.parse().map_err(|_| DbError::InvalidReference {
        reference: s.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::models::Reference;

    #[test]
    fn fmt() {
        vec![
            ("Genesis 50", "Genesis", 50, None),
            ("Joel 2:", "Joel", 2, None),
            ("Song of Solomon 1", "Song of Solomon", 1, None),
            ("Exodus 20", "Exodus", 20, None),
            ("1cor 4", "1cor", 4, None),
            ("John 1:1", "John", 1, Some(1..=1)),
            ("jhn.1.1", "jhn", 1, Some(1..=1)),
            ("I Timothy 3:16", "I Timothy", 3, Some(16..=16)),
            ("1 Timothy 3:16-18", "1 Timothy", 3, Some(16..=18)),
            ("1tim 3.16", "1tim", 3, Some(16..=16)),
        ]
        .iter()
        .for_each(|(raw, book, chapter, verses)| {
            assert_eq!(
                raw.parse::<Reference>().unwrap(),
                Reference {
                    book: (*book).to_string(),
                    chapter: *chapter,
                    verses: verses.to_owned()
                }
            );
        });
    }

    #[test]
    fn from_str() {
        vec![
            ("Genesis 50", "Genesis", 50, None),
            ("Song of Solomon 1", "Song of Solomon", 1, None),
            ("3 John 1", "3 John", 1, None),
            ("Exodus 20", "Exodus", 20, None),
            ("1 Cor 4", "1 Cor", 4, None),
            ("John 1:1", "John", 1, Some(1..=1)),
            ("I Timothy 3:16", "I Timothy", 3, Some(16..=16)),
            ("1 Timothy 3:16-18", "1 Timothy", 3, Some(16..=18)),
            ("1Tim 3:16", "1Tim", 3, Some(16..=16)),
        ]
        .iter()
        .for_each(|(expected, book, chapter, verses)| {
            assert_eq!(
                Reference {
                    book: (*book).to_string(),
                    chapter: *chapter,
                    verses: verses.to_owned()
                }
                .to_string(),
                (*expected).to_string()
            );
        });
    }
}
