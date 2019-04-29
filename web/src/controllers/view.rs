use std::convert::From;

use actix_web::error::BlockingError;
use actix_web::web;
use actix_web::web::HttpResponse;
use actix_web::ResponseError;
use failure::Fail;
use futures::future::{err, Either, Future};
use handlebars::Handlebars;
use lazy_static::lazy_static;
use log::error;
use serde::Serialize;

use db::models::Reference;
use db::{sword_drill, DbError, VerseFormat};

use crate::controllers::*;
use crate::error::Error;
use crate::ServerData;

lazy_static! {
    static ref ERR_TPL: Handlebars = {
        let mut tpl = Handlebars::new();
        tpl.register_template_file("base", "./web/templates/base.hbs")
            .unwrap();
        tpl.register_template_file("error", "./web/templates/error.hbs")
            .unwrap();
        tpl
    };
}

#[derive(Serialize)]
struct TemplatePayload<T: Serialize> {
    data: T,
    meta: Meta,
}

impl<T: Serialize> TemplatePayload<T> {
    /// Create a new HTML template payload.
    fn new(data: T, meta: Meta) -> Self {
        Self { data, meta }
    }

    /// Convert the template payload to HTML
    fn to_html(&self, tpl_name: &str, renderer: &Handlebars) -> Result<String, Error> {
        renderer.render(tpl_name, &self).map_err(|e| {
            error!("{}", e);
            Error::Template
        })
    }
}

#[derive(Fail, Debug)]
#[fail(display = "HTML Error")]
pub struct HtmlError(Error);

impl From<Error> for HtmlError {
    /// Transforms an HtmlError into an actix_web HTTP Response.
    fn from(f: Error) -> Self {
        HtmlError(f)
    }
}

impl ResponseError for HtmlError {
    fn error_response(&self) -> HttpResponse {
        let body = &TemplatePayload::new(ErrorPayload::from_error(&self.0), Meta::for_error())
            .to_html("error", &ERR_TPL)
            .unwrap();

        match self.0 {
            Error::Actix { .. } | Error::Db | Error::Template => {
                HttpResponse::InternalServerError()
            }
            Error::BookNotFound { .. } => HttpResponse::NotFound(),
            Error::InvalidReference { .. } => HttpResponse::BadRequest(),
        }
        .content_type("text/html")
        .body(body)
    }
}

impl From<BlockingError<DbError>> for HtmlError {
    fn from(e: BlockingError<DbError>) -> Self {
        error!("{}", e);
        HtmlError(match e {
            BlockingError::Canceled => Error::Actix {
                cause: e.to_string(),
            },
            BlockingError::Error(db_e) => match db_e {
                DbError::BookNotFound { book } => Error::BookNotFound { book },
                DbError::InvalidReference { reference } => Error::InvalidReference { reference },
                _ => Error::Db,
            },
        })
    }
}

/// Represents an empty payload of data.
///
/// This is used to render Handlebars templates that don't
/// need any context to render (e.g. the About page).
#[derive(Serialize)]
struct EmptyPayload;

/// Handles HTTP requests for the about page.
pub fn about(data: web::Data<ServerData>) -> Result<HttpResponse, HtmlError> {
    let body =
        TemplatePayload::new(EmptyPayload, Meta::for_about()).to_html("about", &data.template)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

/// Handles HTTP requests for a list of all books.
///
/// Return an HTML page that lists all books in the Bible.
pub fn all_books(
    data: web::Data<ServerData>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = HtmlError> {
    let db = data.db.to_owned();
    web::block(move || sword_drill::all_books(&db.get().unwrap()))
        .map_err(HtmlError::from)
        .and_then(move |books| {
            let links = AllBooksLinks {
                books: books.iter().map(|b| book_url(&b.name, &req)).collect(),
            };
            let body = TemplatePayload::new(
                AllBooksPayload {
                    books,
                    links: links.to_owned(),
                },
                Meta::for_all_books(&links),
            )
            .to_html("all-books", &data.template)?;

            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        })
}

/// Handles HTTP requests for a book (e.g. /John)
///
/// Assume the path parameter is a Bible book, and get an HTML response
/// that has book metadata and a list of chapters.
pub fn book(
    data: web::Data<ServerData>,
    path: web::Path<(String,)>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = HtmlError> {
    let db = data.db.to_owned();
    web::block(move || sword_drill::book(&path.0, &db.get().unwrap()))
        .map_err(HtmlError::from)
        .and_then(move |result| {
            let payload = BookPayload::new(result, &req);
            let body =
                TemplatePayload::new(&payload, Meta::for_book(&payload.book, &payload.links))
                    .to_html("book", &data.template)?;

            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        })
}

/// Handles HTTP requests for references (e.g. /John/1/1).
///
/// Parse the URL path for a string that would indicate a reference.
/// If the path parses to a reference, then it is passed to the database
/// layer and looked up, returning an HTTP response with the verse body.
pub fn reference(
    data: web::Data<ServerData>,
    path: web::Path<(String,)>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = HtmlError> {
    let db = data.db.to_owned();
    let raw_reference = path.0.replace("/", ".");
    match raw_reference.parse::<Reference>() {
        Ok(reference) => {
            let payload_reference = reference.to_owned();
            Either::A(
                web::block(move || {
                    sword_drill::verses(&reference, &VerseFormat::HTML, &db.get().unwrap())
                })
                .map_err(HtmlError::from)
                .and_then(move |result| {
                    let payload = VersesPayload::new(result, payload_reference, &req);

                    if payload.verses.is_empty() {
                        Err(Error::InvalidReference {
                            reference: raw_reference,
                        })?;
                    }

                    let body = TemplatePayload::new(
                        &payload,
                        Meta::for_reference(&payload.reference, &payload.verses, &payload.links),
                    )
                    .to_html("chapter", &data.template)?;
                    Ok(HttpResponse::Ok().content_type("text/html").body(body))
                }),
            )
        }
        Err(_) => Either::B(err(HtmlError(Error::InvalidReference {
            reference: raw_reference,
        }))),
    }
}

/// Handle HTTP requests for a search HTML page.
///
/// Return an HTML page with search results based on the `q` query
/// parameter.
pub fn search(
    data: web::Data<ServerData>,
    query: web::Query<SearchParams>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = HtmlError> {
    let db = data.db.to_owned();
    let q = query.q.to_owned();
    web::block(move || sword_drill::search(&query.q, &db.get().unwrap()))
        .map_err(HtmlError::from)
        .and_then(move |result| {
            let body = TemplatePayload::new(
                SearchResultPayload::from_verses_fts(result, &req),
                Meta::for_search(&q, &req.uri().to_string()),
            )
            .to_html("search-results", &data.template)?;
            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        })
}
