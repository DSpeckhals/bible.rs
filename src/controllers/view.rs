use actix_web::{error, HttpResponse, Path, Result, State};
use handlebars::Handlebars;
use serde::Serialize;

use controllers::{ErrorPayload, VersesPayload};
use reference::Reference;
use sword_drill::drill;
use {ReceptusError, ServerState};

#[derive(Serialize)]
struct TemplatePayload<T: Serialize> {
    data: T,
    title: String,
}

impl<T: Serialize> TemplatePayload<T> {
    fn new(title: String, data: T) -> Self {
        Self { title, data }
    }

    fn html(&self, tpl_name: &str, renderer: &Handlebars) -> Result<String, error::Error> {
        renderer
            .render(tpl_name, &self)
            .map_err(error::ErrorInternalServerError)
    }
}

pub fn index(
    (state, info): (State<ServerState>, Path<(String,)>),
) -> Result<HttpResponse, error::Error> {
    let conn = state
        .db
        .get()
        .map_err(|e| ReceptusError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;

    let raw_reference = info.0.replace("/", ".");
    let reference: Reference = raw_reference.parse().map_err(error::ErrorBadRequest)?;

    let payload = VersesPayload::new(
        drill(&reference, &*conn).map_err(|e| match e {
            e @ ReceptusError::BookNotFound { .. } => error::ErrorNotFound(e),
            e => error::ErrorInternalServerError(e),
        })?,
        &reference,
    );

    if payload.verses.is_empty() {
        let e = &ErrorPayload::new(&ReceptusError::InvalidReference {
            reference: reference.to_string(),
        });
        let title = format!("{} Not Found", reference.to_string());
        let body = TemplatePayload::new(title, &e).html("not_found", &state.template)?;
        return Ok(HttpResponse::NotFound()
            .content_type("text/html")
            .body(body));
    }

    let title = format!("The Holy Bible | {}", reference);
    let body = TemplatePayload::new(title, &payload).html("view", &state.template)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
