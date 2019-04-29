use std::convert::From;

use actix_web::error::BlockingError;
use actix_web::web;
use actix_web::web::{HttpRequest, HttpResponse};
use actix_web::ResponseError;
use failure::Fail;
use futures::future::{err, Either, Future};

use db::models::Reference;
use db::{sword_drill, DbError, VerseFormat};

use crate::controllers::{ErrorPayload, SearchParams, SearchResultPayload, VersesPayload};
use crate::error::Error;
use crate::ServerData;

#[derive(Fail, Debug)]
#[fail(display = "Json Error")]
pub struct JsonError(Error);

impl From<Error> for JsonError {
    fn from(f: Error) -> Self {
        JsonError(f)
    }
}

impl From<DbError> for JsonError {
    fn from(_: DbError) -> Self {
        JsonError(Error::Db)
    }
}

impl ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        match self.0 {
            Error::Actix { .. } | Error::Db | Error::Template => {
                HttpResponse::InternalServerError().json(ErrorPayload::from_error(&self.0))
            }
            Error::BookNotFound { .. } => HttpResponse::Ok().json(SearchResultPayload::empty()),
            Error::InvalidReference { .. } => {
                HttpResponse::BadRequest().json(ErrorPayload::from_error(&self.0))
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

pub fn reference(
    data: web::Data<ServerData>,
    path: web::Path<(String,)>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = JsonError> {
    let db = data.db.to_owned();
    match path.0.parse::<Reference>() {
        Ok(reference) => {
            let payload_reference = reference.to_owned();
            Either::A(
                web::block(move || {
                    sword_drill::verses(&reference, &VerseFormat::PlainText, &db.get().unwrap())
                })
                .map_err(JsonError::from)
                .and_then(move |result| {
                    let payload = VersesPayload::new(result, payload_reference, &req);
                    Ok(HttpResponse::Ok().json(payload))
                }),
            )
        }
        Err(e) => Either::B(err(JsonError::from(e))),
    }
}

pub fn search(
    data: web::Data<ServerData>,
    query: web::Query<SearchParams>,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = JsonError> {
    let db = data.db.to_owned();

    // Check if query can be parsed as a reference
    if let Ok(reference) = query.q.parse::<Reference>() {
        Either::A(
            web::block(move || {
                sword_drill::verses(&reference, &VerseFormat::PlainText, &db.get().unwrap())
            })
            .map_err(JsonError::from)
            .and_then(move |results| {
                Ok(HttpResponse::Ok().json(SearchResultPayload::from_verses(results, &req)))
            }),
        )
    // Otherwise look for word matches
    } else {
        Either::B(
            web::block(move || sword_drill::search(&query.q, &db.get().unwrap()))
                .map_err(JsonError::from)
                .and_then(move |results| {
                    Ok(
                        HttpResponse::Ok()
                            .json(SearchResultPayload::from_verses_fts(results, &req)),
                    )
                }),
        )
    }
}
