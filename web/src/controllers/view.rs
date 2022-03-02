use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};

use db::models::Reference;
use db::{SwordDrillable, VerseFormat};

use crate::controllers::SearchParams;
use crate::error::{Error, HtmlError};
use crate::responder::*;
use crate::ServerData;

/// Result for HTML response handlers
type ViewResult = Result<HttpResponse, HtmlError>;

/// Handles HTTP requests for the about page.
pub async fn about(data: web::Data<ServerData>) -> ViewResult {
    let body = TemplateData::new(EmptyData, Meta::for_about()).to_html("about", &data.template)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

/// Handles HTTP requests for a list of all books.
///
/// Return an HTML page that lists all books in the Bible.
pub async fn all_books<SD>(data: web::Data<ServerData>, req: HttpRequest) -> ViewResult
where
    SD: SwordDrillable,
{
    let db = data.db.to_owned();
    let books = web::block(move || SD::all_books(&db.get().unwrap())).await??;

    let books_data = AllBooksData::new(books, &req);
    let body = TemplateData::new(
        books_data.to_owned(),
        Meta::for_all_books(&books_data.links),
    )
    .to_html("all-books", &data.template)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

/// Handles HTTP requests for a book (e.g. /John)
///
/// Assume the path parameter is a Bible book, and get an HTML response
/// that has book metadata and a list of chapters.
pub async fn book<SD>(
    data: web::Data<ServerData>,
    params: web::Path<(String,)>,
    req: HttpRequest,
) -> ViewResult
where
    SD: SwordDrillable,
{
    let (book_name,) = params.into_inner();
    let db = data.db.to_owned();
    let result = web::block(move || SD::book(&book_name, &db.get().unwrap())).await??;
    let book_data = BookData::new(result, &req);
    let body = TemplateData::new(
        &book_data,
        Meta::for_book(&book_data.book, &book_data.links),
    )
    .to_html("book", &data.template)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

/// Handles HTTP requests for references (e.g. /John/1/1).
///
/// Parse the URL path for a string that would indicate a reference.
/// If the path parses to a reference, then it is passed to the database
/// layer and looked up, returning an HTTP response with the verse body.
pub async fn reference<SD>(
    data: web::Data<ServerData>,
    params: web::Path<(String,)>,
    req: HttpRequest,
) -> ViewResult
where
    SD: SwordDrillable,
{
    let (path_reference,) = params.into_inner();
    let db = data.db.to_owned();
    let raw_reference = path_reference.replace("/", ".");

    if let Ok(reference) = raw_reference.parse::<Reference>() {
        let data_reference = reference.to_owned();
        let result =
            web::block(move || SD::verses(&reference, VerseFormat::Html, &db.get().unwrap()))
                .await??;
        let verses_data = VersesData::new(result, data_reference, &req);

        if verses_data.verses.is_empty() {
            return Err(Error::InvalidReference(raw_reference).into());
        }

        let body = TemplateData::new(
            &verses_data,
            Meta::for_reference(
                &verses_data.reference,
                &verses_data.verses,
                &verses_data.links,
            ),
        )
        .to_html("chapter", &data.template)?;
        Ok(HttpResponse::Ok().content_type("text/html").body(body))
    } else {
        Err(Error::InvalidReference(raw_reference).into())
    }
}

/// Handle HTTP requests for a search HTML page.
///
/// Return an HTML page with search results based on the `q` query
/// parameter.
pub async fn search<SD>(
    data: web::Data<ServerData>,
    query: web::Query<SearchParams>,
    req: HttpRequest,
) -> ViewResult
where
    SD: SwordDrillable,
{
    let db = data.db.to_owned();
    let q = query.q.to_owned();
    let result = web::block(move || SD::search(&query.q, &db.get().unwrap())).await??;
    let body = TemplateData::new(
        SearchResultData::from_verses_fts(result, &req),
        Meta::for_search(&q, &req.uri().to_string()),
    )
    .to_html("search-results", &data.template)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[cfg(test)]
mod tests {
    use crate::test::html_response;

    #[test]
    fn about() {
        let result = html_response("/about");
        assert!(result.contains("Where Bible.rs Shines"));
    }

    #[test]
    fn all_books() {
        let result = html_response("/");
        assert!(result.contains("/Psalms"));
    }

    #[test]
    fn book() {
        let result = html_response("/Psalms");
        assert!(result.contains("/Psalms/150"));
    }

    #[test]
    fn reference() {
        let result = html_response("/Psalms/119");
        assert!(result.contains("NUN. Thy word is a lamp unto my feet, and a light unto my path."));
    }
}
