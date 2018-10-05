CREATE TABLE verses (
    id INTEGER PRIMARY KEY NOT NULL,
    book INTEGER NOT NULL REFERENCES books(id),
    chapter INTEGER NOT NULL,
    verse INTEGER NOT NULL,
    words TEXT NOT NULL
);
CREATE INDEX verses_book_chapter_idx ON verses(book, chapter);

INSERT INTO verses
    SELECT CAST(book || substr('000' || chapter, -3, 3) || substr('000' || verse, -3, 3) AS INTEGER) AS id,
        book,
        chapter,
        verse,
        group_concat(
            CASE WHEN open_parentheses THEN '(' ELSE '' END
            || word
            || CASE WHEN close_parentheses THEN ')' ELSE '' END
            || COALESCE(punctuation, ''),
            ' '
        ) AS words
    FROM words
    GROUP BY book, chapter, verse;

CREATE TABLE verses_html (
    id INTEGER PRIMARY KEY NOT NULL,
    book INTEGER NOT NULL REFERENCES books(id),
    chapter INTEGER NOT NULL,
    verse INTEGER NOT NULL,
    words TEXT NOT NULL
);
CREATE INDEX verses_html_book_chapter_idx ON verses_html(book, chapter);

INSERT INTO verses_html
    SELECT CAST(book || substr('000' || chapter, -3, 3) || substr('000' || verse, -3, 3) AS INTEGER) AS id,
        book,
        chapter,
        verse,
        group_concat(
            CASE WHEN open_parentheses THEN '(' ELSE '' END
            || CASE WHEN italic THEN '<em>' ELSE '' END
            || word
            || CASE WHEN italic THEN '</em>' ELSE '' END
            || CASE WHEN close_parentheses THEN ')' ELSE '' END
            || COALESCE(punctuation, ''),
            ' '
        ) AS words
    FROM words
    GROUP BY book, chapter, verse;

CREATE VIRTUAL TABLE verses_fts
USING fts5(book UNINDEXED, chapter UNINDEXED, verse UNINDEXED, words, content="verses", content_rowid="id");

INSERT INTO verses_fts (rowid, book, chapter, verse, words)
SELECT id,
       book,
       chapter,
       verse,
       words
FROM verses
ORDER BY id;
