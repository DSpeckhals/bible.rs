use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use regex::{Match, Regex};

use ReceptusError;

#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    pub book: String,
    pub chapter: Option<i32>,
    pub verses: Option<Range<i32>>,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reference {
                book,
                chapter: None,
                verses: None,
            } => write!(f, "{}", book),
            Reference {
                book,
                chapter: Some(chapter),
                verses: None,
            } => write!(f, "{} {}", book, chapter),
            Reference {
                book,
                chapter: Some(chapter),
                verses: Some(verses),
            } => if verses.start == verses.end {
                write!(f, "{} {}:{}", book, chapter, verses.start)
            } else {
                write!(f, "{} {}:{}-{}", book, chapter, verses.start, verses.end)
            },
            _ => unimplemented!(),
        }
    }
}

impl FromStr for Reference {
    type Err = ReceptusError;

    fn from_str(s: &str) -> Result<Reference, Self::Err> {
        lazy_static! {
            static ref REF_RE: Regex =
                Regex::new(r"^(\w+(?: [a-zA-Z]+(?: [a-zA-Z]+)?)?)(?:\.| )((?:[0-9\-:\.])+)$")
                    .unwrap();
            static ref BOOK_ONLY_RE: Regex = Regex::new(r"^(.*)$").unwrap();
            static ref CV_RE: Regex =
                Regex::new(r"^(\d{1,3})(?:[:\.](\d{1,3})(?:-(\d{1,3}))?)?$").unwrap();
        }

        let ref_caps = REF_RE
            .captures(s)
            .or_else(|| BOOK_ONLY_RE.captures(s))
            .ok_or_else(|| invalid_reference(s))?;
        match (ref_caps.get(1), ref_caps.get(2)) {
            // Only the book
            (Some(book), None) => Ok(Reference {
                book: book.as_str().to_string(),
                chapter: None,
                verses: None,
            }),
            // Book and chapter/verse reference
            (Some(book), Some(cv)) => {
                let cv_caps = CV_RE
                    .captures(cv.as_str())
                    .ok_or_else(|| invalid_reference(s))?;
                let book = book.as_str().to_string();

                match (cv_caps.get(1), cv_caps.get(2), cv_caps.get(3)) {
                    // Chapter only
                    (Some(chapter), None, None) => Ok(Reference {
                        book,
                        chapter: Some(parse_num_match(chapter)?),
                        verses: None,
                    }),
                    // Chapter and one verse
                    (Some(chapter), Some(verse), None) => {
                        let verse = parse_num_match(verse)?;
                        Ok(Reference {
                            book,
                            chapter: Some(parse_num_match(chapter)?),
                            verses: Some(verse..verse),
                        })
                    }
                    // Chapter with more than one verse
                    (Some(chapter), Some(verse_start), Some(verse_end)) => {
                        let verse_start = parse_num_match(verse_start)?;
                        let verse_end = parse_num_match(verse_end)?;
                        Ok(Reference {
                            book,
                            chapter: Some(parse_num_match(chapter)?),
                            verses: Some(verse_start..verse_end),
                        })
                    }
                    _ => Err(invalid_reference(s)),
                }
            }
            _ => Err(invalid_reference(s)),
        }
    }
}

fn parse_num_match(m: Match) -> Result<i32, ReceptusError> {
    m.as_str()
        .parse()
        .map_err(|_| ReceptusError::InvalidReference {
            reference: m.as_str().to_string(),
        })
}

fn invalid_reference(s: &str) -> ReceptusError {
    ReceptusError::InvalidReference {
        reference: s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use reference::Reference;

    #[test]
    fn fmt() {
        vec![
            ("Genesis", "Genesis", None, None),
            ("Song of Solomon", "Song of Solomon", None, None),
            ("Third John", "Third John", None, None),
            ("Exodus 20", "Exodus", Some(20), None),
            ("1cor 4", "1cor", Some(4), None),
            ("John 1:1", "John", Some(1), Some(1..1)),
            ("jhn.1.1", "jhn", Some(1), Some(1..1)),
            ("I Timothy 3:16", "I Timothy", Some(3), Some(16..16)),
            ("1 Timothy 3:16-18", "1 Timothy", Some(3), Some(16..18)),
            ("1tim 3.16", "1tim", Some(3), Some(16..16)),
        ].iter()
        .for_each(|(raw, book, chapter, verses)| {
            assert_eq!(
                raw.parse::<Reference>().unwrap(),
                Reference {
                    book: book.to_string(),
                    chapter: *chapter,
                    verses: verses.clone()
                }
            );
        });
    }

    #[test]
    fn from_str() {
        vec![
            ("Genesis", "Genesis", None, None),
            ("Song of Solomon", "Song of Solomon", None, None),
            ("3 John", "3 John", None, None),
            ("Exodus 20", "Exodus", Some(20), None),
            ("1 Cor 4", "1 Cor", Some(4), None),
            ("John 1:1", "John", Some(1), Some(1..1)),
            ("I Timothy 3:16", "I Timothy", Some(3), Some(16..16)),
            ("1 Timothy 3:16-18", "1 Timothy", Some(3), Some(16..18)),
            ("1Tim 3:16", "1Tim", Some(3), Some(16..16)),
        ].iter()
        .for_each(|(expected, book, chapter, verses)| {
            assert_eq!(
                Reference {
                    book: book.to_string(),
                    chapter: *chapter,
                    verses: verses.clone()
                }.to_string(),
                expected.to_string()
            );
        });
    }
}
