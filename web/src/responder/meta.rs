use serde_derive::{Deserialize, Serialize};

use db::models::{Book, Reference, Verse};

use crate::responder::json_ld::*;
use crate::responder::link::{AllBooksLinks, BookLinks, VersesLinks};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Meta {
    description: String,
    json_ld: Vec<JsonLd>,
    title: String,
    url: String,
}

impl Meta {
    pub fn for_about() -> Self {
        Self {
            description: "About Bible.rs".to_string(),
            json_ld: vec![JsonLd::About(Box::new(AboutJsonLd::new()))],
            title: format!(title_format!(), "About"),
            url: format!(url_format!(), "/about"),
        }
    }

    pub fn for_all_books(links: &AllBooksLinks) -> Self {
        Self {
            description: "Browse and search the King James version of the Bible using a lightning-fast and slick interface.".to_string(),
            json_ld: vec![JsonLd::AllBooks(AllBooksJsonLd::new(links))],
            title: format!(title_format!(), "King James Version"),
            url: format!(url_format!(), ""),
        }
    }

    pub fn for_book(book: &Book, links: &BookLinks) -> Self {
        Self {
            description: format!("The book of {}", book.name),
            json_ld: vec![
                JsonLd::Book(BookJsonLd::new(book, links)),
                JsonLd::BreadcrumbList(BreadcrumbListJsonLd::new(vec![
                    ListItemJsonLd::new(&links.books, 1),
                    ListItemJsonLd::new(&links.current, 2),
                ])),
            ],
            title: format!(title_format!(), book.name),
            url: format!(url_format!(), links.current.url),
        }
    }

    pub fn for_error() -> Self {
        Self {
            description: "Error page".to_string(),
            json_ld: vec![],
            title: format!(title_format!(), "Error"),
            url: format!(url_format!(), ""),
        }
    }

    pub fn for_reference(reference: &Reference, verses: &[Verse], links: &VersesLinks) -> Self {
        let ref_string = reference.to_string();
        Self {
            description: match verses.first() {
                None => ref_string.to_owned(),
                Some(v) => format!("{}...", v.words),
            },
            json_ld: vec![
                JsonLd::Reference(ReferenceJsonLd::new(reference, links)),
                JsonLd::BreadcrumbList(BreadcrumbListJsonLd::new(vec![
                    ListItemJsonLd::new(&links.books, 1),
                    ListItemJsonLd::new(&links.book, 2),
                    ListItemJsonLd::new(&links.current, 3),
                ])),
            ],
            title: format!(title_format!(), ref_string),
            url: format!(url_format!(), links.current.url),
        }
    }

    pub fn for_search(query: &str, url: &str) -> Self {
        let results_string = format!("Results for '{}'", query);
        Self {
            description: results_string.to_owned(),
            json_ld: vec![],
            title: format!(title_format!(), results_string),
            url: format!(url_format!(), url),
        }
    }
}
