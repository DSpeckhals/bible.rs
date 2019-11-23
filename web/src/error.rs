use std::convert::From;

use actix_web::web::HttpResponse;
use actix_web::ResponseError;
use failure::Fail;
use futures::channel::oneshot::Canceled;
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

    #[fail(display = "There was a database error. Root cause: {}", cause)]
    Db { cause: String },

    #[fail(display = "'{}' is not a valid Bible reference.", reference)]
    InvalidReference { reference: String },

    #[fail(display = "There was an error rendering the HTML page.")]
    Template,
}

impl From<DbError> for Error {
    fn from(f: DbError) -> Self {
        match f {
            DbError::InvalidReference { reference } => Error::InvalidReference { reference },
            DbError::BookNotFound { book } => Error::BookNotFound { book },
            DbError::Other { cause } => Error::Db {
                cause: cause.to_string(),
            },
            _ => Error::Db {
                cause: f.to_string(),
            },
        }
    }
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
    fn from(f: DbError) -> Self {
        JsonError(f.into())
    }
}

impl From<Canceled> for JsonError {
    fn from(f: Canceled) -> Self {
        JsonError(Error::Actix {
            cause: f.to_string(),
        })
    }
}

impl ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        match &self.0 {
            Error::Actix { .. } | Error::Template => {
                error!("Unhandled: {}", &self.0);
                HttpResponse::InternalServerError().json(ErrorData::from_error(&self.0))
            }
            Error::Db { cause } => {
                error!("Database error: {}", cause);
                HttpResponse::InternalServerError().json(ErrorData::new(cause))
            }
            Error::BookNotFound { .. } => HttpResponse::Ok().json(SearchResultData::empty()),
            Error::InvalidReference { .. } => {
                HttpResponse::BadRequest().json(ErrorData::from_error(&self.0))
            }
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
    fn from(f: Error) -> Self {
        HtmlError(f)
    }
}

impl From<DbError> for HtmlError {
    fn from(f: DbError) -> Self {
        HtmlError(f.into())
    }
}

impl From<Canceled> for HtmlError {
    fn from(f: Canceled) -> Self {
        HtmlError(Error::Actix {
            cause: f.to_string(),
        })
    }
}

impl ResponseError for HtmlError {
    fn error_response(&self) -> HttpResponse {
        let body = &TemplateData::new(ErrorData::from_error(&self.0), Meta::for_error())
            .to_html("error", &ERR_TPL)
            .unwrap();

        match self.0 {
            Error::Actix { .. } | Error::Db { .. } | Error::Template => {
                error!("Unhandled: {}", &self.0);
                HttpResponse::InternalServerError()
            }
            Error::BookNotFound { .. } => HttpResponse::NotFound(),
            Error::InvalidReference { .. } => HttpResponse::BadRequest(),
        }
        .content_type("text/html")
        .body(body)
    }
}
