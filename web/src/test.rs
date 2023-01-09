use std::str;

use actix_web::{test, web, App, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;

use db::models::*;
use db::*;

use crate::ServerData;
use crate::{api, view};

pub async fn with_service<F>(f: F)
where
    F: Fn(HttpRequest) + Clone + 'static,
{
    let srv = test::init_service(
        App::new()
            .service(web::resource("/test").to(move |req: HttpRequest| {
                f(req);
                HttpResponse::Ok()
            }))
            .service(web::resource("/").name("bible"))
            .service(web::resource("{book}").name("book"))
            .service(web::resource("{reference:.+\\d}").name("reference")),
    );

    test::call_service(
        &srv.await,
        test::TestRequest::with_uri("/test").to_request(),
    )
    .await;
}

fn test_book() -> Book {
    Book {
        id: 19,
        name: "Psalms".to_string(),
        chapter_count: 150,
        testament: Testament::Old,
    }
}

pub struct TestSwordDrill;

impl SwordDrillable for TestSwordDrill {
    fn verses(
        _: &Reference,
        _: VerseFormat,
        _: &mut DbConnection,
    ) -> Result<(Book, Vec<Verse>), DbError> {
        let book = test_book();

        let verse = Verse {
            id: 555,
            book: 19,
            chapter: 119,
            verse: 105,
            words: "NUN. Thy word is a lamp unto my feet, and a light unto my path.".to_string(),
        };

        Ok((book, vec![verse]))
    }

    fn book(_: &str, _: &mut DbConnection) -> Result<(Book, Vec<i32>), DbError> {
        Ok((test_book(), (1..=150).collect()))
    }

    fn all_books(_: &mut DbConnection) -> Result<Vec<Book>, DbError> {
        Ok(vec![test_book()])
    }

    fn search(_: &str, _: &mut DbConnection) -> Result<Vec<(VerseFTS, Book)>, DbError> {
        let book = test_book();
        let verse = VerseFTS {
            book: 19,
            chapter: 119,
            verse: 105,
            words: "NUN. Thy word is a lamp unto my feet, and a <em>light</em> unto my path."
                .to_string(),
            rank: 1.1,
        };

        Ok(vec![(verse, book)])
    }
}

pub async fn json_response<T>(uri: &str) -> T
where
    T: DeserializeOwned,
{
    let srv = test::init_service(
        App::new()
            .app_data(web::Data::new(ServerData {
                books: BOOKS.to_vec(),
                db: build_pool(":memory:"),
                template: Handlebars::default(),
            }))
            .service(web::resource("/").name("bible"))
            .service(web::resource("{book}").name("book"))
            .service(web::resource("{reference:.+\\d}").name("reference"))
            .service(web::resource("api/search").to(api::search::<TestSwordDrill>))
            .service(web::resource("api/{reference}.json").to(api::reference::<TestSwordDrill>)),
    );

    let req = test::TestRequest::with_uri(uri).to_request();
    test::call_and_read_body_json(&srv.await, req).await
}

pub async fn html_response(uri: &str) -> String {
    let mut template = Handlebars::new();
    template.set_strict_mode(true);
    template
        .register_templates_directory(".hbs", "./templates/")
        .expect("Could not register template files");

    let srv = test::init_service(
        App::new()
            .app_data(web::Data::new(ServerData {
                books: BOOKS.to_vec(),
                db: build_pool(":memory:"),
                template,
            }))
            .service(web::resource("about").to(view::about))
            .service(
                web::resource("/")
                    .name("bible")
                    .to(view::all_books::<TestSwordDrill>),
            )
            .service(
                web::resource("{book}")
                    .name("book")
                    .to(view::book::<TestSwordDrill>),
            )
            .service(
                web::resource("{reference:.+\\d}")
                    .name("reference")
                    .to(view::reference::<TestSwordDrill>),
            )
            .service(web::resource("api/search").to(api::search::<TestSwordDrill>))
            .service(web::resource("api/{reference}.json").to(api::reference::<TestSwordDrill>)),
    );

    let req = test::TestRequest::with_uri(uri).to_request();

    str::from_utf8(&test::call_and_read_body(&srv.await, req).await)
        .expect("Could not convert response to UTF8")
        .to_string()
}

lazy_static! {
    pub static ref BOOKS: Vec<Book> = [
        ("Genesis", 50),
        ("Exodus", 40),
        ("Leviticus", 27),
        ("Numbers", 36),
        ("Deuteronomy", 34),
        ("Joshua", 24),
        ("Judges", 21),
        ("Ruth", 4),
        ("1 Samuel", 31),
        ("2 Samuel", 24),
        ("1 Kings", 22),
        ("2 Kings", 25),
        ("1 Chronicles", 29),
        ("2 Chronicles", 36),
        ("Ezra", 10),
        ("Nehemiah", 13),
        ("Esther", 10),
        ("Job", 42),
        ("Psalms", 150),
        ("Proverbs", 31),
        ("Ecclesiastes", 12),
        ("Song of Solomon", 8),
        ("Isaiah", 66),
        ("Jeremiah", 52),
        ("Lamentations", 5),
        ("Ezekiel", 48),
        ("Daniel", 12),
        ("Hosea", 14),
        ("Joel", 3),
        ("Amos", 9),
        ("Obadiah", 1),
        ("Jonah", 4),
        ("Micah", 7),
        ("Nahum", 3),
        ("Habakkuk", 3),
        ("Zephaniah", 3),
        ("Haggai", 2),
        ("Zechariah", 14),
        ("Malachi", 4),
        ("Matthew", 28),
        ("Mark", 16),
        ("Luke", 24),
        ("John", 21),
        ("Acts", 28),
        ("Romans", 16),
        ("1 Corinthians", 16),
        ("2 Corinthians", 13),
        ("Galatians", 6),
        ("Ephesians", 6),
        ("Philippians", 4),
        ("Colossians", 4),
        ("1 Thessalonians", 5),
        ("2 Thessalonians", 3),
        ("1 Timothy", 6),
        ("2 Timothy", 4),
        ("Titus", 3),
        ("Philemon", 1),
        ("Hebrews", 13),
        ("James", 5),
        ("1 Peter", 5),
        ("2 Peter", 3),
        ("1 John", 5),
        ("2 John", 1),
        ("3 John", 1),
        ("Jude", 1),
        ("Revelation", 22),
    ]
    .iter()
    .enumerate()
    .map(|(i, (name, chapter_count))| Book {
        id: (i + 1) as i32,
        name: name.to_string(),
        chapter_count: *chapter_count,
        testament: if i < 40 {
            Testament::Old
        } else {
            Testament::New
        },
    })
    .collect();
}
