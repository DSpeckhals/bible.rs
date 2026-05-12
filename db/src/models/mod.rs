use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql, FromSqlRow, Queryable};
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};

/// Model representing a Bible verse.
#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
pub struct Verse {
    pub id: i32,
    pub book: i32,
    pub chapter: i32,
    pub verse: i32,
    pub words: String,
}

/// Enum for the testaments in the Bible (Old or New). This is mapped
/// to a column in the database table `books`.
#[derive(Clone, Debug, Deserialize, Serialize, FromSqlRow)]
pub enum Testament {
    Old,
    New,
}

impl<DB> FromSql<Text, DB> for Testament
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "OLD" => Ok(Testament::Old),
            "NEW" => Ok(Testament::New),
            _ => Err("Unexpected testament in the Bible".into()),
        }
    }
}

/// Model representing a book in the Bible.
#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub chapter_count: i32,
    pub testament: Testament,
}

/// Model representing a Bible book's abbreviation.
#[derive(Clone, Debug, Deserialize, Queryable)]
pub struct BookAbbreviation {
    pub id: i32,
    pub book_id: i32,
    pub abbreviation: String,
}

/// Model representing a full text search Bible verse.
#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
pub struct VerseFTS {
    pub book: i32,
    pub chapter: i32,
    pub verse: i32,
    pub words: String,
    pub rank: f32,
}

mod reference;
pub use self::reference::Reference;
