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

table! {
    verses_html (id) {
        id -> Integer,
        book -> Integer,
        chapter -> Integer,
        verse -> Integer,
        words -> Text,
    }
}

allow_tables_to_appear_in_same_query!(books, verses_fts);

mod auto;
