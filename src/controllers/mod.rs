pub mod api {
    use actix_web::{error, HttpResponse, Json, Path, Result, State};
    use serde_json;

    use models::{Book, Verse};
    use reference::Reference;
    use sword_drill::drill;
    use {ReceptusError, ServerState};

    #[derive(Serialize)]
    pub struct VersesPayload {
        book: Book,
        reference: String,
        verses: Vec<Verse>,
    }

    impl VersesPayload {
        pub fn new((book, verses): (Book, Vec<Verse>), reference: &Reference) -> Self {
            Self {
                book,
                reference: reference.to_string(),
                verses,
            }
        }
    }

    #[derive(Serialize)]
    struct ErrorPayload {
        message: String,
    }

    impl ErrorPayload {
        pub fn new(e: &ReceptusError) -> Self {
            Self {
                message: e.to_string(),
            }
        }

        pub fn json(&self) -> String {
            serde_json::to_string_pretty(&self).unwrap()
        }
    }

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

    pub fn index(
        (state, info): (State<ServerState>, Path<(String,)>),
    ) -> Result<Json<VersesPayload>> {
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
}
