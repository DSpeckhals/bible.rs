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
        chapter_count -> Integer,
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

joinable!(book_abbreviations -> books (book_id));
joinable!(verses -> books (book));

allow_tables_to_appear_in_same_query!(
    book_abbreviations,
    books,
    verses,
);
