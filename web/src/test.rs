use std::str;

use actix_rt::System;
use actix_web::{test, web, App, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use serde::de::DeserializeOwned;

use db::models::*;
use db::*;

use crate::ServerData;
use crate::{api, view};

pub fn with_service<F>(f: F)
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

    System::new("test").block_on(async move {
        test::call_service(
            &mut srv.await,
            test::TestRequest::with_uri("/test").to_request(),
        )
        .await
    });
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
        _: &VerseFormat,
        _: &DbConnection,
    ) -> Result<(Book, Vec<Verse>), DbError> {
        let book = test_book();

        let verse = Verse {
            id: 555,
            book: 19,
            chapter: 1,
            verse: 105,
            words: "NUN. Thy word is a lamp unto my feet, and a light unto my path.".to_string(),
        };

        Ok((book, vec![verse]))
    }

    fn book(_: &str, _: &DbConnection) -> Result<(Book, Vec<i32>), DbError> {
        Ok((test_book(), (1..=150).collect()))
    }

    fn all_books(_: &DbConnection) -> Result<Vec<Book>, DbError> {
        Ok(vec![test_book()])
    }

    fn search(_: &str, _: &DbConnection) -> Result<Vec<(VerseFTS, Book)>, DbError> {
        let book = test_book();
        let verse = VerseFTS {
            book: 19,
            chapter: 1,
            verse: 105,
            words: "NUN. Thy word is a lamp unto my feet, and a <em>light</em> unto my path."
                .to_string(),
            rank: 1.1,
        };

        Ok(vec![(verse, book)])
    }
}

pub fn json_response<T>(uri: &str) -> T
where
    T: DeserializeOwned,
{
    let srv = test::init_service(
        App::new()
            .data(ServerData {
                db: build_pool(":memory:"),
                template: Handlebars::default(),
            })
            .service(web::resource("/").name("bible"))
            .service(web::resource("{book}").name("book"))
            .service(web::resource("{reference:.+\\d}").name("reference"))
            .service(web::resource("api/search").to(api::search::<TestSwordDrill>))
            .service(web::resource("api/{reference}.json").to(api::reference::<TestSwordDrill>)),
    );

    let req = test::TestRequest::with_uri(uri).to_request();

    System::new("test").block_on(async move { test::read_response_json(&mut srv.await, req).await })
}

pub fn html_response(uri: &str) -> String {
    let mut template = Handlebars::new();
    template.set_strict_mode(true);
    template
        .register_templates_directory(".hbs", "./templates/")
        .expect("Could not register template files");

    let srv = test::init_service(
        App::new()
            .data(ServerData {
                db: build_pool(":memory:"),
                template,
            })
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

    System::new("test").block_on(async move {
        str::from_utf8(&test::read_response(&mut srv.await, req).await)
            .expect("Could not convert response to UTF8")
            .to_string()
    })
}
