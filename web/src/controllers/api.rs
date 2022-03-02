use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};

use db::models::Reference;
use db::{SwordDrillable, VerseFormat};

use crate::controllers::SearchParams;
use crate::error::{Error, JsonError};
use crate::responder::{SearchResultData, VersesData};
use crate::ServerData;

/// Result for JSON API response handlers
type ApiResult = Result<HttpResponse, JsonError>;

pub async fn reference<SD>(
    data: web::Data<ServerData>,
    params: web::Path<(String,)>,
    req: HttpRequest,
) -> ApiResult
where
    SD: SwordDrillable,
{
    let (path_reference,) = params.into_inner();
    let raw_reference = path_reference.replace("/", ".");

    if let Ok(reference) = raw_reference.parse::<Reference>() {
        let data_reference = reference.to_owned();
        let result = web::block(move || {
            SD::verses(&reference, VerseFormat::PlainText, &data.db.get().unwrap())
        })
        .await??;

        let verses_data = VersesData::new(result, data_reference, &req);
        Ok(HttpResponse::Ok().json(verses_data))
    } else {
        Err(Error::InvalidReference(raw_reference).into())
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
    if let Ok(reference) = query.q.parse::<Reference>() {
        let results = web::block(move || {
            SD::verses(&reference, VerseFormat::PlainText, &data.db.get().unwrap())
        })
        .await??;
        Ok(HttpResponse::Ok().json(SearchResultData::from_verses(results, &req)))
    } else {
        let results = web::block(move || SD::search(&query.q, &data.db.get().unwrap())).await??;
        Ok(HttpResponse::Ok().json(SearchResultData::from_verses_fts(results, &req)))
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
