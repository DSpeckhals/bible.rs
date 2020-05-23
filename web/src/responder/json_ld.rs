use serde::ser;
use serde_derive::{Deserialize, Serialize};

use db::models::{Book, Reference};

use crate::responder::link::{AllBooksLinks, BookLinks, Link, VersesLinks, NAME};

const CONTEXT: &str = "https://schema.org";
const CREATOR_FIRST_NAME: &str = "Dustin";
const CREATOR_LAST_NAME: &str = "Speckhals";
const CREATOR_URL: &str = "https://speckhals.com";
const LANGUAGE: &str = "en-us";
const KEYWORDS: &str = "bible,kjv";
const VERSION: &str = "King James Version";

#[derive(Clone, Deserialize, Debug)]
pub enum JsonLd {
    BreadcrumbList(BreadcrumbListJsonLd),
    About(Box<AboutJsonLd>),
    AllBooks(AllBooksJsonLd),
    Book(BookJsonLd),
    Reference(ReferenceJsonLd),
}

impl ser::Serialize for JsonLd {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(
            &match self {
                JsonLd::BreadcrumbList(s) => serde_json::to_string_pretty(&s),
                JsonLd::About(s) => serde_json::to_string_pretty(&s),
                JsonLd::AllBooks(s) => serde_json::to_string_pretty(&s),
                JsonLd::Book(s) => serde_json::to_string_pretty(&s),
                JsonLd::Reference(s) => serde_json::to_string_pretty(&s),
            }
            .unwrap(),
        )
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
enum Kind {
    BookSeries,
    Book,
    BreadcrumbList,
    Chapter,
    ListItem,
    Person,
    Thing,
    Website,
}

/********** json-ld Building Blocks **********/

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BreadcrumbListJsonLd {
    #[serde(rename = "@context")]
    context: String,

    item_list_element: Vec<ListItemJsonLd>,

    #[serde(rename = "@type")]
    kind: Kind,
}

impl BreadcrumbListJsonLd {
    pub(super) fn new(list_items: Vec<ListItemJsonLd>) -> Self {
        Self {
            context: CONTEXT.to_string(),
            item_list_element: list_items,
            kind: Kind::BreadcrumbList,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListItemJsonLd {
    item: ThingJsonLd,

    #[serde(rename = "@type")]
    kind: Kind,

    name: String,
    position: i32,
}

impl ListItemJsonLd {
    pub(super) fn new(link: &Link, position: i32) -> Self {
        Self {
            item: ThingJsonLd {
                id: format!(url_format!(), link.url),
                name: link.label.to_owned(),
                url: format!(url_format!(), link.url),
                kind: match position {
                    1 => Kind::BookSeries,
                    2 => Kind::Book,
                    3 => Kind::Chapter,
                    _ => Kind::Thing,
                },
                ..ThingJsonLd::default()
            },
            kind: Kind::ListItem,
            name: link.label.to_owned(),
            position,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct PartJsonLd {
    #[serde(rename = "@id")]
    id: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PersonJsonLd {
    family_name: String,
    given_name: String,

    #[serde(flatten)]
    thing: ThingJsonLd,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ThingJsonLd {
    #[serde(rename = "@context")]
    context: String,

    #[serde(rename = "@id")]
    id: String,

    #[serde(rename = "@type")]
    kind: Kind,

    name: String,
    url: String,
}

impl Default for ThingJsonLd {
    fn default() -> Self {
        Self {
            context: CONTEXT.to_string(),
            id: format!(url_format!(), ""),
            kind: Kind::Thing,
            name: "Default".to_string(),
            url: format!(url_format!(), ""),
        }
    }
}

/********** Page Implementations **********/

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AboutJsonLd {
    creator: PersonJsonLd,
    keywords: String,

    #[serde(flatten)]
    thing: ThingJsonLd,
}

impl AboutJsonLd {
    pub(super) fn new() -> Self {
        let person_thing = ThingJsonLd {
            id: CREATOR_URL.to_string(),
            kind: Kind::Person,
            name: format!("{} {}", CREATOR_FIRST_NAME, CREATOR_LAST_NAME),
            url: CREATOR_URL.to_string(),
            ..ThingJsonLd::default()
        };
        let creator = PersonJsonLd {
            thing: person_thing,
            family_name: CREATOR_LAST_NAME.to_string(),
            given_name: CREATOR_FIRST_NAME.to_string(),
        };
        let thing = ThingJsonLd {
            id: format!(url_format!(), "/about"),
            kind: Kind::Website,
            name: NAME.to_string(),
            url: format!(url_format!(), "/about"),
            ..ThingJsonLd::default()
        };

        Self {
            creator,
            keywords: KEYWORDS.to_string(),
            thing,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AllBooksJsonLd {
    has_part: Vec<PartJsonLd>,
    in_language: String,

    #[serde(flatten)]
    thing: ThingJsonLd,

    version: String,
}

impl AllBooksJsonLd {
    pub(super) fn new(links: &AllBooksLinks) -> Self {
        let has_part = links
            .books
            .iter()
            .map(|b| PartJsonLd {
                id: format!(url_format!(), b.url),
            })
            .collect();
        let thing = ThingJsonLd {
            id: format!(url_format!(), ""),
            kind: Kind::BookSeries,
            name: NAME.to_string(),
            url: format!(url_format!(), ""),
            ..ThingJsonLd::default()
        };

        Self {
            has_part,
            in_language: LANGUAGE.to_string(),
            thing,
            version: VERSION.to_string(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BookJsonLd {
    has_part: Vec<PartJsonLd>,
    in_language: String,
    is_part_of: PartJsonLd,
    position: i32,

    #[serde(flatten)]
    thing: ThingJsonLd,
}

impl BookJsonLd {
    pub(super) fn new(book: &Book, links: &BookLinks) -> Self {
        let has_part = links
            .chapters
            .iter()
            .map(|c| PartJsonLd {
                id: format!(url_format!(), c),
            })
            .collect();
        let is_part_of = PartJsonLd {
            id: format!(url_format!(), links.books.url),
        };
        let thing = ThingJsonLd {
            id: format!(url_format!(), links.current.url),
            kind: Kind::Book,
            name: book.name.to_owned(),
            url: format!(url_format!(), links.current.url),
            ..ThingJsonLd::default()
        };

        Self {
            has_part,
            in_language: LANGUAGE.to_string(),
            is_part_of,
            position: book.id,
            thing,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceJsonLd {
    is_part_of: PartJsonLd,
    position: i32,

    #[serde(flatten)]
    thing: ThingJsonLd,
}

impl ReferenceJsonLd {
    pub(super) fn new(reference: &Reference, links: &VersesLinks) -> Self {
        let thing = ThingJsonLd {
            id: format!(url_format!(), links.current.url),
            kind: Kind::Chapter,
            name: reference.to_string(),
            url: format!(url_format!(), links.current.url),
            ..ThingJsonLd::default()
        };
        let is_part_of = PartJsonLd {
            id: format!(url_format!(), links.book.url),
        };

        Self {
            is_part_of,
            position: reference.chapter,
            thing,
        }
    }
}
