pub use self::auto::*;

table! {
    verses_fts (rowid) {
        rowid -> Integer,
        book -> Integer,
        chapter -> Integer,
        verse -> Integer,
        #[sql_name = "verses_fts"]
        text -> Text,
        words -> Text,
        rank -> Float,
    }
}

allow_tables_to_appear_in_same_query!(books, verses_fts);

mod auto;
