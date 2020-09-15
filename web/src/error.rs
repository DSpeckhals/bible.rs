use std::convert::From;

use actix_web::error::BlockingError;
use actix_web::web::HttpResponse;
use actix_web::ResponseError;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use log::error;
use thiserror::Error;

use db::DbError;

use crate::responder::{ErrorData, Meta, SearchResultData, TemplateData};

/// Error type for the Bible.rs application.
#[derive(Clone, Error, Debug)]
pub enum Error {
    #[error("There was an error with Actix. Cause: {0}")]
    Actix(String),

    #[error("'{0}' was not found.")]
    BookNotFound(String),

    #[error("There was a database error. Root cause: {0}")]
    Db(String),

    #[error("{0} is not a valid Bible reference.")]
    InvalidReference(String),

    #[error("There was an error rendering the HTML page.")]
    Template,
}

impl From<DbError> for Error {
    fn from(f: DbError) -> Self {
        match f {
            DbError::InvalidReference { reference } => Error::InvalidReference(reference),
            DbError::BookNotFound { book } => Error::BookNotFound(book),
            DbError::Migration { cause }
            | DbError::Other { cause }
            | DbError::ConnectionPool { cause } => Error::Db(cause),
        }
    }
}

#[derive(Error, Debug)]
#[error("Error: {0}")]
/// Error to display as JSON
pub struct JsonError(#[from] pub Error);

// impl From<DbError> for JsonError {
//     fn from(f: DbError) -> Self {
//         JsonError(f.into())
//     }
// }

impl ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        match &self.0 {
            Error::Actix { .. } | Error::Template => {
                error!("Unhandled: {}", &self.0);
                HttpResponse::InternalServerError().json(ErrorData::from_error(&self.0))
            }
            Error::Db(cause) => {
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

impl From<BlockingError<DbError>> for JsonError {
    fn from(f: BlockingError<DbError>) -> Self {
        JsonError(match f {
            BlockingError::Canceled => Error::Actix(f.to_string()),
            BlockingError::Error(db_e) => db_e.into(),
        })
    }
}

lazy_static! {
    static ref ERR_TPL: Handlebars<'static> = {
        let mut tpl = Handlebars::new();
        tpl.register_template_file("base", "./web/templates/base.hbs")
            .unwrap();
        tpl.register_template_file("error", "./web/templates/error.hbs")
            .unwrap();
        tpl
    };
}

#[derive(Error, Debug)]
#[error("Error: {0}")]
/// Error to display as HTML.
pub struct HtmlError(#[from] pub Error);

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

impl From<BlockingError<DbError>> for HtmlError {
    fn from(f: BlockingError<DbError>) -> Self {
        HtmlError(match f {
            BlockingError::Canceled => Error::Actix(f.to_string()),
            BlockingError::Error(db_e) => db_e.into(),
        })
    }
}
