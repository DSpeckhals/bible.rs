table! {
    book_abbreviations (id) {
        id -> Integer,
        book_id -> Integer,
        abbreviation -> Text,
    }
}

table! {
    books (id) {
        id -> Integer,
        name -> Text,
        testament -> Text,
    }
}

table! {
    verses (id) {
        id -> Integer,
        book -> Integer,
        chapter -> Integer,
        verse -> Integer,
        words -> Text,
    }
}

joinable!(verses -> books (book));

allow_tables_to_appear_in_same_query!(
    book_abbreviations,
    books,
    verses,
);
