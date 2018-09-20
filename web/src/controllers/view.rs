use actix_web::{error, FromRequest, HttpRequest, HttpResponse, Path, Result, State};
use handlebars::Handlebars;
use serde::Serialize;

use db::models::Reference;
use db::{sword_drill, BiblersError};

use controllers::{AllBooksPayload, BookPayload, ErrorPayload, SearchResultPayload, VersesPayload};
use ServerState;

lazy_static! {
    static ref ERR_TPL: Handlebars = {
        let mut tpl = Handlebars::new();
        tpl.register_template_file("base", "./web/templates/base.hbs")
            .unwrap();
        tpl.register_template_file("error", "./web/templates/error.hbs")
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
pub struct HtmlBiblersError(BiblersError);

impl From<BiblersError> for HtmlBiblersError {
    fn from(f: BiblersError) -> Self {
        HtmlBiblersError(f)
    }
}

impl error::ResponseError for HtmlBiblersError {
    fn error_response(&self) -> HttpResponse {
        let body = &TemplatePayload::new("Error".to_string(), ErrorPayload::new(&self.0))
            .html("error", &ERR_TPL)
            .unwrap();

        match self.0 {
            BiblersError::BookNotFound { .. } => HttpResponse::NotFound()
                .content_type("text/html")
                .body(body),
            BiblersError::ConnectionPoolError { .. } => HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body),
            BiblersError::DatabaseError { .. } => HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body),
            BiblersError::DatabaseMigrationError { .. } => HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body),
            BiblersError::InvalidReference { .. } => HttpResponse::BadRequest()
                .content_type("text/html")
                .body(body),
            BiblersError::TemplateError => HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(body),
        }
    }
}

macro_rules! title_format {
    () => {
        "Bible.rs | {}"
    };
}

pub fn all_books((state,): (State<ServerState>,)) -> Result<HttpResponse, HtmlBiblersError> {
    let conn = state
        .db
        .get()
        .map_err(|e| BiblersError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;

    let books = sword_drill::all_books(&*conn)?;
    let title = format!(title_format!(), "King James Version");
    let body = TemplatePayload::new(title, AllBooksPayload { books })
        .html("all-books", &state.template)
        .map_err(|e| {
            error!("{:?}", e);
            BiblersError::TemplateError
        })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn book(req: &HttpRequest<ServerState>) -> Result<HttpResponse, HtmlBiblersError> {
    let info = Path::<(String,)>::extract(req).unwrap();
    let conn = req
        .state()
        .db
        .get()
        .map_err(|e| BiblersError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;

    let result = sword_drill::book(&info.0, &*conn)?;
    let title = format!(title_format!(), result.0.name);
    let body = TemplatePayload::new(title, BookPayload::new(result, &req.drop_state()))
        .html("book", &req.state().template)
        .map_err(|e| {
            error!("{:?}", e);
            BiblersError::TemplateError
        })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn index(req: &HttpRequest<ServerState>) -> Result<HttpResponse, HtmlBiblersError> {
    let info = Path::<(String,)>::extract(req).unwrap();
    let conn = req
        .state()
        .db
        .get()
        .map_err(|e| BiblersError::ConnectionPoolError {
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
        Err(BiblersError::InvalidReference {
            reference: payload.reference.to_string(),
        })?;
    }

    let title = format!(title_format!(), payload.reference.to_string());
    let body = TemplatePayload::new(title, payload)
        .html("chapter", &req.state().template)
        .map_err(|e| {
            error!("{:?}", e);
            BiblersError::TemplateError
        })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn search(req: &HttpRequest<ServerState>) -> Result<HttpResponse, HtmlBiblersError> {
    let conn = req
        .state()
        .db
        .get()
        .map_err(|e| BiblersError::ConnectionPoolError {
            root_cause: e.to_string(),
        })?;
    let params = req.query();
    let q = params.get("q").ok_or(BiblersError::TemplateError)?;
    let title = format!(title_format!(), format!("Results for '{}'", q));
    let results = sword_drill::search(q, &conn)?;
    let body = TemplatePayload::new(
        title,
        SearchResultPayload::from_verses_fts(results, &req.drop_state()),
    ).html("search-results", &req.state().template)
    .map_err(|e| {
        error!("{:?}", e);
        BiblersError::TemplateError
    })?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
