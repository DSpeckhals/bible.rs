use std::convert::From;

use actix_web::{error, FromRequest, HttpRequest, HttpResponse, Json, Path, Result};

use controllers::{ErrorPayload, SearchResultPayload, VersesPayload};
use reference::Reference;
use sword_drill;
use {ReceptusError, ServerState};

#[derive(Fail, Debug)]
#[fail(display = "Json Error")]
pub struct JsonReceptusError(ReceptusError);

impl From<ReceptusError> for JsonReceptusError {
    fn from(f: ReceptusError) -> Self {
        JsonReceptusError(f)
    }
}

impl error::ResponseError for JsonReceptusError {
    fn error_response(&self) -> HttpResponse {
        match self.0 {
            ref e @ ReceptusError::BookNotFound { .. } => {
                HttpResponse::NotFound().body(ErrorPayload::new(e).json())
            }
            ref e @ ReceptusError::ConnectionPoolError { .. } => {
                HttpResponse::InternalServerError().body(ErrorPayload::new(e).json())
            }
            ref e @ ReceptusError::DatabaseError { .. } => {
                HttpResponse::InternalServerError().body(ErrorPayload::new(e).json())
            }
            ref e @ ReceptusError::InvalidReference { .. } => {
                HttpResponse::BadRequest().body(ErrorPayload::new(e).json())
            }
            ref e @ ReceptusError::TemplateError => {
                HttpResponse::InternalServerError().body(ErrorPayload::new(e).json())
            }
        }
    }
}

pub fn index(req: &HttpRequest<ServerState>) -> Result<Json<VersesPayload>, JsonReceptusError> {
    let info = Path::<(String,)>::extract(req).unwrap();
    let conn = req
        .state()
        .db
        .get()
        .map_err(|e| ReceptusError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;
    let reference: Reference = info.0.parse()?;
    let payload = VersesPayload::new(
        sword_drill::verses(&reference, &*conn)?,
        reference,
        &req.drop_state(),
    );
    Ok(Json(payload))
}

pub fn search(
    req: &HttpRequest<ServerState>,
) -> Result<Json<SearchResultPayload>, JsonReceptusError> {
    let conn = req
        .state()
        .db
        .get()
        .map_err(|e| ReceptusError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;

    req.query()
        .get("q")
        .map_or(Ok(Json(SearchResultPayload::empty())), |q| {
            // Check if query can be parsed as a reference
            if let Ok(r) = q.parse::<Reference>() {
                match sword_drill::verses(&r, &conn) {
                    Ok(results) => Ok(Json(SearchResultPayload::from_verses(
                        results,
                        &req.drop_state(),
                    ))),
                    Err(ReceptusError::BookNotFound { .. })
                    | Err(ReceptusError::InvalidReference { .. }) => {
                        Ok(Json(SearchResultPayload::empty()))
                    }
                    Err(e) => Err(JsonReceptusError::from(e)),
                }
            // Otherwise look for word matches
            } else {
                Ok(Json(SearchResultPayload::from_verses_fts(
                    sword_drill::search(q, &conn)?,
                    &req.drop_state(),
                )))
            }
        })
}
