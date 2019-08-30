use std::convert::From;

use actix_web::error::BlockingError;
use actix_web::web::HttpResponse;
use actix_web::ResponseError;
use failure::Fail;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use log::error;

use db::DbError;

use crate::responder::{ErrorData, Meta, SearchResultData, TemplateData};

/// Error type that for the Bible.rs application.
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(
        display = "There was an error with the Actix async arbiter. Cause: {}",
        cause
    )]
    Actix { cause: String },

    #[fail(display = "'{}' was not found.", book)]
    BookNotFound { book: String },

    #[fail(display = "There was a database error.")]
    Db,

    #[fail(display = "'{}' is not a valid Bible reference.", reference)]
    InvalidReference { reference: String },

    #[fail(display = "There was an error rendering the HTML page.")]
    Template,
}

#[derive(Fail, Debug)]
#[fail(display = "Error: {}", _0)]
/// Error to display as JSON
pub struct JsonError(Error);

impl From<Error> for JsonError {
    fn from(f: Error) -> Self {
        JsonError(f)
    }
}

impl From<DbError> for JsonError {
    fn from(e: DbError) -> Self {
        JsonError(match e {
            DbError::InvalidReference { reference } => Error::InvalidReference { reference },
            DbError::BookNotFound { book } => Error::BookNotFound { book },
            _ => Error::Db {},
        })
    }
}

impl ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        match self.0 {
            Error::Actix { .. } | Error::Db | Error::Template => {
                error!("Unhandled: {}", &self.0);
                HttpResponse::InternalServerError().json(ErrorData::from_error(&self.0))
            }
            Error::BookNotFound { .. } => HttpResponse::Ok().json(SearchResultData::empty()),
            Error::InvalidReference { .. } => {
                HttpResponse::BadRequest().json(ErrorData::from_error(&self.0))
            }
        }
    }
}

impl From<BlockingError<DbError>> for JsonError {
    fn from(e: BlockingError<DbError>) -> Self {
        match e {
            BlockingError::Canceled => JsonError(Error::Actix {
                cause: e.to_string(),
            }),
            BlockingError::Error(db_e) => match db_e {
                DbError::BookNotFound { book } => JsonError(Error::BookNotFound { book }),
                _ => JsonError(Error::Db),
            },
        }
    }
}

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

#[derive(Fail, Debug)]
#[fail(display = "HTML Error")]
/// Error to display as HTML.
pub struct HtmlError(pub Error);

impl From<Error> for HtmlError {
    /// Transforms an HtmlError into an actix_web HTTP Response.
    fn from(f: Error) -> Self {
        HtmlError(f)
    }
}

impl ResponseError for HtmlError {
    fn error_response(&self) -> HttpResponse {
        let body = &TemplateData::new(ErrorData::from_error(&self.0), Meta::for_error())
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
