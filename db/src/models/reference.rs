use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use regex::{Match, Regex};

use DbError;

/// Model representing a Bible reference used to look up a
/// passage in the database.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Reference {
    pub book: String,
    pub chapter: i32,
    pub verses: Option<Range<i32>>,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
            } => if verses.start == verses.end {
                write!(f, "{} {}:{}", book, chapter, verses.start)
            } else {
                write!(f, "{} {}:{}-{}", book, chapter, verses.start, verses.end)
            },
        }
    }
}

impl FromStr for Reference {
    type Err = DbError;

    fn from_str(s: &str) -> Result<Reference, Self::Err> {
        lazy_static! {
            static ref REF_RE: Regex =
                Regex::new(r"^(\w+(?: [a-zA-Z]+(?: [a-zA-Z]+)?)?)(?:\.| )((?:[0-9\-:\.])+)$")
                    .unwrap();
            static ref CV_RE: Regex =
                Regex::new(r"^(\d{1,3})(?:[:\.](\d{1,3})?(?:-(\d{1,3}))?)?$").unwrap();
        }

        let ref_caps = REF_RE.captures(s).ok_or_else(|| invalid_reference(s))?;
        match (ref_caps.get(1), ref_caps.get(2)) {
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
                        chapter: parse_num_match(chapter)?,
                        verses: None,
                    }),
                    // Chapter and one verse
                    (Some(chapter), Some(verse), None) => {
                        let verse = parse_num_match(verse)?;
                        Ok(Reference {
                            book,
                            chapter: parse_num_match(chapter)?,
                            verses: Some(verse..verse),
                        })
                    }
                    // Chapter with more than one verse
                    (Some(chapter), Some(verse_start), Some(verse_end)) => {
                        let verse_start = parse_num_match(verse_start)?;
                        let verse_end = parse_num_match(verse_end)?;
                        Ok(Reference {
                            book,
                            chapter: parse_num_match(chapter)?,
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

/// Parse a [Match](regex.Match.html) into an i32.
fn parse_num_match(m: Match) -> Result<i32, DbError> {
    m.as_str()
        .parse()
        .map_err(|_| DbError::InvalidReferenceError {
            reference: m.as_str().to_string(),
        })
}

/// Create an invalid reference error from the input.
fn invalid_reference(s: &str) -> DbError {
    DbError::InvalidReferenceError {
        reference: s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use models::Reference;

    #[test]
    fn fmt() {
        vec![
            ("Genesis 50", "Genesis", 50, None),
            ("Joel 2:", "Joel", 2, None),
            ("Song of Solomon 1", "Song of Solomon", 1, None),
            ("Exodus 20", "Exodus", 20, None),
            ("1cor 4", "1cor", 4, None),
            ("John 1:1", "John", 1, Some(1..1)),
            ("jhn.1.1", "jhn", 1, Some(1..1)),
            ("I Timothy 3:16", "I Timothy", 3, Some(16..16)),
            ("1 Timothy 3:16-18", "1 Timothy", 3, Some(16..18)),
            ("1tim 3.16", "1tim", 3, Some(16..16)),
        ].iter()
        .for_each(|(raw, book, chapter, verses)| {
            assert_eq!(
                raw.parse::<Reference>().unwrap(),
                Reference {
                    book: book.to_string(),
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
            ("John 1:1", "John", 1, Some(1..1)),
            ("I Timothy 3:16", "I Timothy", 3, Some(16..16)),
            ("1 Timothy 3:16-18", "1 Timothy", 3, Some(16..18)),
            ("1Tim 3:16", "1Tim", 3, Some(16..16)),
        ].iter()
        .for_each(|(expected, book, chapter, verses)| {
            assert_eq!(
                Reference {
                    book: book.to_string(),
                    chapter: *chapter,
                    verses: verses.to_owned()
                }.to_string(),
                expected.to_string()
            );
        });
    }
}
