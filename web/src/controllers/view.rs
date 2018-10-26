use std::convert::From;

use actix_web::*;
use actix_web::actix::*;
use futures::future::{err, Future};
use handlebars::Handlebars;
use serde::Serialize;

use db::models::Reference;
use db::VerseFormat;

use actors::*;
use controllers::*;
use error::Error;
use ServerState;

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

impl error::ResponseError for HtmlError {
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
        }.content_type("text/html")
        .body(body)
    }
}

impl From<MailboxError> for HtmlError {
    fn from(e: MailboxError) -> Self {
        HtmlError(Error::Actix {
            cause: e.to_string(),
        })
    }
}

type AsyncResponse = Box<Future<Item = HttpResponse, Error = HtmlError>>;

/// Represents an empty payload of data.
///
/// This is used to render Handlebars templates that don't
/// need any context to render (e.g. the About page).
#[derive(Serialize)]
struct EmptyPayload;

/// Handles HTTP requests for the about page.
pub fn about((state,): (State<ServerState>,)) -> Result<HttpResponse, HtmlError> {
    let body =
        TemplatePayload::new(EmptyPayload, Meta::for_about()).to_html("about", &state.template)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

/// Handles HTTP requests for a list of all books.
///
/// Return an HTML page that lists all books in the Bible.
pub fn all_books((state,): (State<ServerState>,)) -> AsyncResponse {
    state
        .db
        .send(AllBooksMessage)
        .from_err()
        .and_then(move |res| match res {
            Ok(books) => {
                let body = TemplatePayload::new(AllBooksPayload { books }, Meta::for_all_books())
                    .to_html("all-books", &state.template)?;

                Ok(HttpResponse::Ok().content_type("text/html").body(body))
            }
            Err(e) => Err(HtmlError(e)),
        }).responder()
}

/// Handles HTTP requests for a book (e.g. /John)
///
/// Assume the path parameter is a Bible book, and get an HTML response
/// that has book metadata and a list of chapters.
pub fn book(req: &HttpRequest<ServerState>) -> AsyncResponse {
    let info = Path::<(String,)>::extract(req).unwrap();
    let db = &req.state().db;

    let req = req.to_owned();
    db.send(BookMessage {
        name: info.0.to_owned(),
    }).from_err()
    .and_then(move |res| match res {
        Ok(result) => {
            let payload = BookPayload::new(result, &req.drop_state());
            let body = TemplatePayload::new(
                &payload,
                Meta::for_book(&payload.book, &payload.links.current.url),
            ).to_html("book", &req.state().template)?;

            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        }
        Err(e) => Err(HtmlError(e)),
    }).responder()
}

/// Handles HTTP requests for references (e.g. /John/1/1).
///
/// Parse the URL path for a string that would indicate a reference.
/// If the path parses to a reference, then it is passed to the database
/// layer and looked up, returning an HTTP response with the verse body.
pub fn reference(req: &HttpRequest<ServerState>) -> AsyncResponse {
    let info = Path::<(String,)>::extract(req).unwrap();
    let raw_reference = info.0.replace("/", ".");
    let reference = match raw_reference.parse::<Reference>() {
        Ok(r) => r,
        Err(_) => {
            return Box::new(err(HtmlError(Error::InvalidReference {
                reference: raw_reference,
            })))
        }
    };

    let db = &req.state().db;
    let req = req.to_owned();
    db.send(VersesMessage {
        reference: reference.to_owned(),
        format: VerseFormat::HTML,
    }).from_err()
    .and_then(move |res| match res {
        Ok(result) => {
            let payload = VersesPayload::new(result, reference, &req.drop_state());

            if payload.verses.is_empty() {
                Err(Error::InvalidReference {
                    reference: raw_reference,
                })?;
            }

            let body = TemplatePayload::new(
                &payload,
                Meta::for_reference(
                    &payload.reference,
                    &payload.verses,
                    &payload.links.current.url,
                ),
            ).to_html("chapter", &req.state().template)?;
            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        }
        Err(e) => Err(HtmlError(e)),
    }).responder()
}

/// Handle HTTP requests for a search HTML page.
///
/// Return an HTML page with search results based on the `q` query
/// parameter.
pub fn search(req: &HttpRequest<ServerState>) -> AsyncResponse {
    let params = req.query();
    let query = match params.get("q") {
        Some(q) => q,
        None => {
            return Box::new(err(HtmlError(Error::InvalidReference {
                reference: "".to_string(),
            })))
        }
    }.to_owned();

    let db = &req.state().db;
    let req = req.to_owned();
    db.send(SearchMessage {
        query: query.to_owned(),
    }).from_err()
    .and_then(move |res| match res {
        Ok(result) => {
            let body = TemplatePayload::new(
                SearchResultPayload::from_verses_fts(result, &req.drop_state()),
                Meta::for_search(&query, &req.uri().to_string()),
            ).to_html("search-results", &req.state().template)?;
            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        }
        Err(e) => Err(HtmlError(e)),
    }).responder()
}
