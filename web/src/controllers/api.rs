use actix_web::web;
use actix_web::web::{HttpRequest, HttpResponse};

use db::models::Reference;
use db::{SwordDrillable, VerseFormat};

use crate::controllers::SearchParams;
use crate::error::JsonError;
use crate::responder::{SearchResultData, VersesData};
use crate::ServerData;

/// Result for JSON API response handlers
type ApiResult = Result<HttpResponse, JsonError>;

pub async fn reference<SD>(
    data: web::Data<ServerData>,
    path: web::Path<(String,)>,
    req: HttpRequest,
) -> ApiResult
where
    SD: SwordDrillable,
{
    let db = data.db.to_owned();
    match path.0.parse::<Reference>() {
        Ok(reference) => {
            let data_reference = reference.to_owned();
            let result = web::block(move || {
                SD::verses(&reference, &VerseFormat::PlainText, &db.get().unwrap())
            })
            .await?;

            let verses_data = VersesData::new(result, data_reference, &req);
            Ok(HttpResponse::Ok().json(verses_data))
        }
        Err(e) => Err(JsonError::from(e)),
    }
}

pub async fn search<SD>(
    data: web::Data<ServerData>,
    query: web::Query<SearchParams>,
    req: HttpRequest,
) -> ApiResult
where
    SD: SwordDrillable,
{
    let db = data.db.to_owned();

    // Check if query can be parsed as a reference
    match query.q.parse() {
        Ok(reference) => {
            let results = web::block(move || {
                SD::verses(&reference, &VerseFormat::PlainText, &db.get().unwrap())
            })
            .await?;
            Ok(HttpResponse::Ok().json(SearchResultData::from_verses(results, &req)))
        }
        Err(_) => {
            let results = web::block(move || SD::search(&query.q, &db.get().unwrap())).await?;
            Ok(HttpResponse::Ok().json(SearchResultData::from_verses_fts(results, &req)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::responder::{SearchResultData, VersesData};
    use crate::test::json_response;

    #[test]
    fn reference() {
        let result: VersesData = json_response("/api/psalms.119.105.json");
        assert_eq!(
            result.verses[0].words,
            "NUN. Thy word is a lamp unto my feet, and a light unto my path."
        );
    }

    #[test]
    fn search() {
        // By words
        let result: SearchResultData = json_response("/api/search?q=word");
        assert_eq!(
            result.matches[0].text,
            "NUN. Thy word is a lamp unto my feet, and a <em>light</em> unto my path."
        );

        // By reference
        let result: SearchResultData = json_response("/api/search?q=psalms%20119:105");
        assert_eq!(
            result.matches[0].text,
            "NUN. Thy word is a lamp unto my feet, and a light unto my path."
        );
    }
}
