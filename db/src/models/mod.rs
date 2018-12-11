use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql, FromSqlRow, Queryable};
use diesel::row::Row;
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use serde_derive::Serialize;

/// Model representing a Bible verse.
#[derive(Clone, Debug, Queryable, Serialize)]
pub struct Verse {
    pub id: i32,
    pub book: i32,
    pub chapter: i32,
    pub verse: i32,
    pub words: String,
}

/// Enum for the testaments in the Bible (Old or New). This is mapped
/// to a column in the database table `books`.
#[derive(Clone, Debug, Serialize)]
pub enum Testament {
    Old,
    New,
}

impl FromSql<Text, Sqlite> for Testament {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        let testament =
            <String as FromSql<Text, Sqlite>>::from_sql(bytes).expect("Unexpected null testament");
        match testament.as_ref() {
            "OLD" => Ok(Testament::Old),
            "NEW" => Ok(Testament::New),
            _ => Err("Unexpected testament in the Bible".into()),
        }
    }
}

impl FromSqlRow<Text, Sqlite> for Testament {
    fn build_from_row<T: Row<Sqlite>>(row: &mut T) -> deserialize::Result<Self> {
        FromSql::<Text, Sqlite>::from_sql(row.take())
    }
}

impl Queryable<Text, Sqlite> for Testament {
    type Row = Self;

    fn build(row: Self::Row) -> Self {
        row
    }
}

/// Model representing a book in the Bible.
#[derive(Clone, Debug, Queryable, Serialize)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub chapter_count: i32,
    pub testament: Testament,
}

/// Model representing a Bible book's abbreviation.
#[derive(Clone, Debug, Queryable)]
pub struct BookAbbreviation {
    pub id: i32,
    pub book_id: i32,
    pub abbreviation: String,
}

/// Model representing a full text search Bible verse.
#[derive(Clone, Debug, Queryable, Serialize)]
pub struct VerseFTS {
    pub book: i32,
    pub chapter: i32,
    pub verse: i32,
    pub words: String,
    pub rank: f32,
}

mod reference;
pub use self::reference::Reference;
