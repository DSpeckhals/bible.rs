use std::convert::From;

use actix_web::{error, FromRequest, HttpRequest, HttpResponse, Json, Path, Result};

use controllers::{ErrorPayload, VersesPayload};
use reference::Reference;
use sword_drill::verses;
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
    let payload = VersesPayload::new(verses(&reference, &*conn)?, reference, &req.drop_state());
    Ok(Json(payload))
}
