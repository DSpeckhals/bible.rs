use std::convert::From;

use actix::prelude::*;
use actix_web::*;
use futures::future::{err, ok, Future};

use db::models::Reference;
use db::{DbError, VerseFormat};

use actors::{SearchMessage, VersesMessage};
use controllers::{ErrorPayload, SearchResultPayload, VersesPayload};
use error::BiblersError;
use ServerState;

#[derive(Fail, Debug)]
#[fail(display = "Json Error")]
pub struct JsonBiblersError(BiblersError);

impl From<BiblersError> for JsonBiblersError {
    fn from(f: BiblersError) -> Self {
        JsonBiblersError(f)
    }
}

impl error::ResponseError for JsonBiblersError {
    fn error_response(&self) -> HttpResponse {
        match self.0 {
            BiblersError::TemplateError | BiblersError::DbError => {
                HttpResponse::InternalServerError()
            }
        }.body(ErrorPayload::from_error(&self.0).to_json())
    }
}

impl From<MailboxError> for JsonBiblersError {
    fn from(_: MailboxError) -> Self {
        JsonBiblersError(BiblersError::TemplateError)
    }
}

pub fn reference(
    req: &HttpRequest<ServerState>,
) -> Box<Future<Item = Json<VersesPayload>, Error = JsonBiblersError>> {
    let db = &req.state().db;
    let info = Path::<(String,)>::extract(req).unwrap();
    let reference = info.0.parse::<Reference>();
    if reference.is_err() {
        return Box::new(err(JsonBiblersError::from(BiblersError::DbError)));
    }
    let reference = reference.unwrap();

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
        Err(_) => Err(JsonBiblersError::from(BiblersError::DbError)),
    }).responder()
}

pub fn search(
    req: &HttpRequest<ServerState>,
) -> Box<Future<Item = Json<SearchResultPayload>, Error = JsonBiblersError>> {
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
                    Err(DbError::BookNotFoundError { .. }) => {
                        Ok(Json(SearchResultPayload::empty()))
                    }
                    Err(_) => Err(JsonBiblersError::from(BiblersError::DbError)),
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
                    Err(_) => Err(JsonBiblersError::from(BiblersError::DbError)),
                }).responder()
            }
        })
}
