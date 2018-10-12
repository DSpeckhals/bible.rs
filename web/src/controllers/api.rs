use std::convert::From;

use actix_web::{error, FromRequest, HttpRequest, HttpResponse, Json, Path, Result};

use db::models::Reference;
use db::sword_drill;
use db::{DbError, VerseFormat};

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

pub fn reference(req: &HttpRequest<ServerState>) -> Result<Json<VersesPayload>, JsonBiblersError> {
    let info = Path::<(String,)>::extract(req).unwrap();
    let conn = req.state().db.get().map_err(|_| BiblersError::DbError)?;
    let reference: Reference = info.0.parse().map_err(|_| BiblersError::DbError)?;

    let payload = VersesPayload::new(
        sword_drill::verses(&reference, &VerseFormat::PlainText, &*conn)
            .map_err(|_| BiblersError::DbError)?,
        reference,
        &req.drop_state(),
    );
    Ok(Json(payload))
}

pub fn search(
    req: &HttpRequest<ServerState>,
) -> Result<Json<SearchResultPayload>, JsonBiblersError> {
    let conn = req.state().db.get().map_err(|_| BiblersError::DbError)?;

    req.query()
        .get("q")
        .map_or(Ok(Json(SearchResultPayload::empty())), |q| {
            // Check if query can be parsed as a reference
            if let Ok(r) = q.parse::<Reference>() {
                match sword_drill::verses(&r, &VerseFormat::PlainText, &conn) {
                    Ok(results) => Ok(Json(SearchResultPayload::from_verses(
                        results,
                        &req.drop_state(),
                    ))),
                    Err(DbError::BookNotFoundError { .. }) => {
                        Ok(Json(SearchResultPayload::empty()))
                    }
                    Err(_) => Err(JsonBiblersError::from(BiblersError::DbError)),
                }
            // Otherwise look for word matches
            } else {
                Ok(Json(SearchResultPayload::from_verses_fts(
                    sword_drill::search(q, &conn).map_err(|_| BiblersError::DbError)?,
                    &req.drop_state(),
                )))
            }
        })
}
