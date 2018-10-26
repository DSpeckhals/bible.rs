use std::convert::From;

use actix_web::*;
use actix_web::actix::*;
use futures::future::{err, ok, Future};

use db::models::Reference;
use db::{DbError, VerseFormat};

use actors::{SearchMessage, VersesMessage};
use controllers::{ErrorPayload, SearchResultPayload, VersesPayload};
use error::Error;
use ServerState;

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

impl error::ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        match self.0 {
            Error::Actix { .. } | Error::Db | Error::Template => {
                HttpResponse::InternalServerError()
            }
            Error::BookNotFound { .. } => HttpResponse::NotFound(),
            Error::InvalidReference { .. } => HttpResponse::BadRequest(),
        }.body(ErrorPayload::from_error(&self.0).to_json())
    }
}

impl From<MailboxError> for JsonError {
    fn from(_: MailboxError) -> Self {
        JsonError(Error::Db)
    }
}

pub fn reference(
    req: &HttpRequest<ServerState>,
) -> Box<Future<Item = Json<VersesPayload>, Error = JsonError>> {
    let db = &req.state().db;
    let info = Path::<(String,)>::extract(req).unwrap();
    let reference = match info.0.parse::<Reference>() {
        Ok(r) => r,
        Err(e) => return Box::new(err(JsonError::from(e))),
    };

    let req = req.to_owned();
    db.send(VersesMessage {
        reference: reference.to_owned(),
        format: VerseFormat::PlainText,
    }).from_err()
    .and_then(move |res| match res {
        Ok(result) => {
            let payload = VersesPayload::new(result, reference, &req.drop_state());
            Ok(Json(payload))
        }
        Err(e) => Err(JsonError::from(e)),
    }).responder()
}

pub fn search(
    req: &HttpRequest<ServerState>,
) -> Box<Future<Item = Json<SearchResultPayload>, Error = JsonError>> {
    let db = &req.state().db;
    req.query()
        .get("q")
        .map_or(Box::new(ok(Json(SearchResultPayload::empty()))), |q| {
            // Check if query can be parsed as a reference
            if let Ok(reference) = q.parse::<Reference>() {
                let req = req.to_owned();
                db.send(VersesMessage {
                    reference,
                    format: VerseFormat::PlainText,
                }).from_err()
                .and_then(move |res| match res {
                    Ok(results) => Ok(Json(SearchResultPayload::from_verses(
                        results,
                        &req.drop_state(),
                    ))),
                    Err(Error::BookNotFound { .. }) => Ok(Json(SearchResultPayload::empty())),
                    Err(e) => Err(JsonError::from(e)),
                }).responder()
            // Otherwise look for word matches
            } else {
                let req = req.to_owned();
                db.send(SearchMessage {
                    query: q.to_owned(),
                }).from_err()
                .and_then(move |res| match res {
                    Ok(results) => Ok(Json(SearchResultPayload::from_verses_fts(
                        results,
                        &req.drop_state(),
                    ))),
                    Err(e) => Err(JsonError::from(e)),
                }).responder()
            }
        })
}
