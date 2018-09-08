use actix_web::{error, HttpResponse, Json, Path, Result, State};

use controllers::{ErrorPayload, VersesPayload};
use reference::Reference;
use sword_drill::drill;
use {ReceptusError, ServerState};

impl error::ResponseError for ReceptusError {
    fn error_response(&self) -> HttpResponse {
        match self {
            e @ ReceptusError::BookNotFound { .. } => {
                HttpResponse::NotFound().body(ErrorPayload::new(e).json())
            }
            e @ ReceptusError::ConnectionPoolError { .. } => {
                HttpResponse::InternalServerError().body(ErrorPayload::new(e).json())
            }
            e @ ReceptusError::DatabaseError { .. } => {
                HttpResponse::InternalServerError().body(ErrorPayload::new(e).json())
            }
            e @ ReceptusError::InvalidReference { .. } => {
                HttpResponse::BadRequest().body(ErrorPayload::new(e).json())
            }
        }
    }
}

pub fn index((state, info): (State<ServerState>, Path<(String,)>)) -> Result<Json<VersesPayload>> {
    let conn = state
        .db
        .get()
        .map_err(|e| ReceptusError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;
    let reference: Reference = info.0.parse()?;
    let payload = VersesPayload::new(drill(&reference, &*conn)?, &reference);
    Ok(Json(payload))
}
