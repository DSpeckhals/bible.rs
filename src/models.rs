use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql, FromSqlRow, Queryable};
use diesel::row::Row;
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;

#[derive(Debug, Queryable, Serialize)]
pub struct Verse {
    pub id: i32,
    pub book: i32,
    pub chapter: i32,
    pub verse: i32,
    pub words: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Queryable, Serialize)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub chapter_count: i32,
    pub testament: Testament,
}

#[derive(Debug, Queryable)]
pub struct BookAbbreviation {
    pub id: i32,
    pub book_id: i32,
    pub abbreviation: String,
}
