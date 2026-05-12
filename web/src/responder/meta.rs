use serde_derive::{Deserialize, Serialize};

use db::models::{Book, Reference, Verse};

use crate::responder::json_ld::*;
use crate::responder::link::{AllBooksLinks, BookLinks, Link, NAME, VersesLinks};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Meta {
    description: String,
    json_ld: Vec<JsonLd>,
    title: String,
    url: String,
}

fn bible_root_link() -> Link {
    Link {
        label: NAME.to_string(),
        url: "/".to_string(),
    }
}

impl Meta {
    pub fn for_about() -> Self {
        let root = bible_root_link();
        let here = Link {
            label: "About".to_string(),
            url: "/about".to_string(),
        };
        Self {
            description: "About Bible.rs".to_string(),
            json_ld: vec![
                JsonLd::About(Box::new(AboutJsonLd::new())),
                JsonLd::BreadcrumbList(BreadcrumbListJsonLd::new(vec![
                    ListItemJsonLd::new(&root, 1, BREADCRUMB_SITE),
                    ListItemJsonLd::new(&here, 2, BREADCRUMB_ABOUT),
                ])),
            ],
            title: format!(title_format!(), "About"),
            url: format!(url_format!(), "/about"),
        }
    }

    pub fn for_all_books(links: &AllBooksLinks) -> Self {
        Self {
            description: "Browse and search the King James version of the Bible using a lightning-fast, ad-free reader.".to_string(),
            json_ld: vec![
                JsonLd::WebSite(WebSiteJsonLd::new()),
                JsonLd::AllBooks(AllBooksJsonLd::new(links)),
            ],
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
                    ListItemJsonLd::new(&links.books, 1, BREADCRUMB_SITE),
                    ListItemJsonLd::new(&links.current, 2, BREADCRUMB_BOOK),
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
                    ListItemJsonLd::new(&links.books, 1, BREADCRUMB_SITE),
                    ListItemJsonLd::new(&links.book, 2, BREADCRUMB_BOOK),
                    ListItemJsonLd::new(&links.current, 3, BREADCRUMB_CHAPTER),
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
            json_ld: vec![JsonLd::SearchResults(SearchResultsPageJsonLd::new(
                query, url,
            ))],
            title: format!(title_format!(), results_string),
            url: format!(url_format!(), url),
        }
    }
}
