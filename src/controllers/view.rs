use actix_web::{error, FromRequest, HttpRequest, HttpResponse, Path, Result, State};
use handlebars::Handlebars;
use serde::Serialize;

use controllers::{AllBooksPayload, BookPayload, ErrorPayload, VersesPayload};
use reference::Reference;
use sword_drill;
use {ReceptusError, ServerState};

lazy_static! {
    static ref ERR_TPL: Handlebars = {
        let mut tpl = Handlebars::new();
        tpl.register_template_file("base", "./templates/base.hbs")
            .unwrap();
        tpl.register_template_file("error", "./templates/error.hbs")
            .unwrap();
        tpl
    };
}

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

#[derive(Fail, Debug)]
#[fail(display = "HTML Error")]
pub struct HtmlReceptusError(ReceptusError);

impl From<ReceptusError> for HtmlReceptusError {
    fn from(f: ReceptusError) -> Self {
        HtmlReceptusError(f)
    }
}

impl error::ResponseError for HtmlReceptusError {
    fn error_response(&self) -> HttpResponse {
        let body = &TemplatePayload::new("Error".to_string(), ErrorPayload::new(&self.0))
            .html("error", &ERR_TPL)
            .unwrap();

        match self.0 {
            ReceptusError::BookNotFound { .. } => HttpResponse::NotFound()
                .content_type("text/html")
                .body(body),
            ReceptusError::ConnectionPoolError { .. } => HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body),
            ReceptusError::DatabaseError { .. } => HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body),
            ReceptusError::InvalidReference { .. } => HttpResponse::BadRequest()
                .content_type("text/html")
                .body(body),
            ReceptusError::TemplateError => HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body),
        }
    }
}

macro_rules! title_format {
    () => {
        "The Holy Bible | {}"
    };
}

pub fn index(req: &HttpRequest<ServerState>) -> Result<HttpResponse, HtmlReceptusError> {
    let info = Path::<(String,)>::extract(req).unwrap();
    let conn = req
        .state()
        .db
        .get()
        .map_err(|e| ReceptusError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;

    let raw_reference = info.0.replace("/", ".");
    let reference: Reference = raw_reference.parse()?;

    let payload = VersesPayload::new(
        sword_drill::verses(&reference, &*conn)?,
        reference, // Consume the reference, thus making it no longer usable
        &req.drop_state(),
    );

    if payload.verses.is_empty() {
        Err(ReceptusError::InvalidReference {
            reference: payload.reference.to_string(),
        })?;
    }

    let title = format!(title_format!(), payload.reference.to_string());
    let body = TemplatePayload::new(title, payload)
        .html("chapter", &req.state().template)
        .map_err(|e| {
            error!("{:?}", e);
            ReceptusError::TemplateError
        })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn book(req: &HttpRequest<ServerState>) -> Result<HttpResponse, HtmlReceptusError> {
    let info = Path::<(String,)>::extract(req).unwrap();
    let conn = req
        .state()
        .db
        .get()
        .map_err(|e| ReceptusError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;

    let result = sword_drill::book(&info.0, &*conn)?;
    let title = format!(title_format!(), result.0.name);
    let body = TemplatePayload::new(title, BookPayload::new(result, &req.drop_state()))
        .html("book", &req.state().template)
        .map_err(|e| {
            error!("{:?}", e);
            ReceptusError::TemplateError
        })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn all_books((state,): (State<ServerState>,)) -> Result<HttpResponse, HtmlReceptusError> {
    let conn = state
        .db
        .get()
        .map_err(|e| ReceptusError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;

    let books = sword_drill::all_books(&*conn)?;
    let title = format!(title_format!(), "King James Version");
    let body = TemplatePayload::new(title, AllBooksPayload { books })
        .html("all-books", &state.template)
        .map_err(|e| {
            error!("{:?}", e);
            ReceptusError::TemplateError
        })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
