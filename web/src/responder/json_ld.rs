use serde::ser;
use serde_derive::{Deserialize, Serialize};

use db::models::{Book, Reference};

use crate::responder::link::{AllBooksLinks, BookLinks, Link, NAME, VersesLinks};

const CONTEXT: &str = "https://schema.org";
const CREATOR_FIRST_NAME: &str = "Dustin";
const CREATOR_LAST_NAME: &str = "Speckhals";
const CREATOR_URL: &str = "https://speckhals.com";
const LANGUAGE: &str = "en-us";
const KEYWORDS: &str = "bible,kjv";
const VERSION: &str = "King James Version";

/// Wraps each emitted JSON-LD blob. `Serialize` renders the variant as a
/// pretty-printed JSON *string*, which is what the Handlebars template
/// inlines verbatim into a `<script type="application/ld+json">` block.
#[derive(Clone, Deserialize, Debug)]
pub enum JsonLd {
    About(Box<AboutJsonLd>),
    AllBooks(AllBooksJsonLd),
    Book(BookJsonLd),
    BreadcrumbList(BreadcrumbListJsonLd),
    Reference(ReferenceJsonLd),
    SearchResults(SearchResultsPageJsonLd),
    WebSite(WebSiteJsonLd),
}

impl ser::Serialize for JsonLd {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(
            &match self {
                JsonLd::About(s) => serde_json::to_string_pretty(&s),
                JsonLd::AllBooks(s) => serde_json::to_string_pretty(&s),
                JsonLd::Book(s) => serde_json::to_string_pretty(&s),
                JsonLd::BreadcrumbList(s) => serde_json::to_string_pretty(&s),
                JsonLd::Reference(s) => serde_json::to_string_pretty(&s),
                JsonLd::SearchResults(s) => serde_json::to_string_pretty(&s),
                JsonLd::WebSite(s) => serde_json::to_string_pretty(&s),
            }
            .unwrap(),
        )
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub(super) enum Kind {
    AboutPage,
    Book,
    BookSeries,
    BreadcrumbList,
    Chapter,
    EntryPoint,
    ListItem,
    Person,
    SearchAction,
    SearchResultsPage,
    Thing,
    #[serde(rename = "WebSite")]
    Website,
}

/********** Shared building blocks **********/

/// A minimal node with `@id`, `@type`, and `name` — used for `hasPart`
/// and `isPartOf` references inside the larger JSON-LD graph. Crawlers
/// can resolve these to richer nodes elsewhere on the site.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PartJsonLd {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    kind: Kind,
    name: String,
}

/// Carries `@context` — used at the root of every top-level JSON-LD blob.
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

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PersonJsonLd {
    family_name: String,
    given_name: String,

    #[serde(flatten)]
    thing: ThingJsonLd,
}

/// Breadcrumb leaf node — no `@context` (the surrounding BreadcrumbList
/// already declares it). This is the fix for the leak that was happening
/// when `ThingJsonLd` was used here non-flattened.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BreadcrumbItem {
    #[serde(rename = "@id")]
    id: String,

    #[serde(rename = "@type")]
    kind: Kind,

    name: String,
    url: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListItemJsonLd {
    item: BreadcrumbItem,

    #[serde(rename = "@type")]
    kind: Kind,

    name: String,
    position: i32,
}

impl ListItemJsonLd {
    pub(super) fn new(link: &Link, position: i32, item_kind: Kind) -> Self {
        Self {
            item: BreadcrumbItem {
                id: format!(url_format!(), link.url),
                kind: item_kind,
                name: link.label.to_owned(),
                url: format!(url_format!(), link.url),
            },
            kind: Kind::ListItem,
            name: link.label.to_owned(),
            position,
        }
    }
}

/// Type aliases meta.rs uses to label breadcrumb leaves with their
/// schema.org type, without exposing the full `Kind` enum.
pub(super) const BREADCRUMB_SITE: Kind = Kind::Website;
pub(super) const BREADCRUMB_BOOK: Kind = Kind::Book;
pub(super) const BREADCRUMB_CHAPTER: Kind = Kind::Chapter;
pub(super) const BREADCRUMB_ABOUT: Kind = Kind::AboutPage;

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

/// Search action — enables Google's sitelinks search box on the home
/// page when emitted under a `WebSite` node's `potentialAction`.
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchActionJsonLd {
    #[serde(rename = "@type")]
    kind: Kind,

    target: EntryPointJsonLd,

    #[serde(rename = "query-input")]
    query_input: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntryPointJsonLd {
    #[serde(rename = "@type")]
    kind: Kind,

    url_template: String,
}

impl SearchActionJsonLd {
    fn new() -> Self {
        Self {
            kind: Kind::SearchAction,
            target: EntryPointJsonLd {
                kind: Kind::EntryPoint,
                url_template: format!(url_format!(), "/search?q={search_term_string}"),
            },
            query_input: "required name=search_term_string".to_string(),
        }
    }
}

/********** Page-level types **********/

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebSiteJsonLd {
    in_language: String,

    #[serde(flatten)]
    thing: ThingJsonLd,

    potential_action: SearchActionJsonLd,
}

impl WebSiteJsonLd {
    pub(super) fn new() -> Self {
        Self {
            in_language: LANGUAGE.to_string(),
            thing: ThingJsonLd {
                id: format!(url_format!(), "/"),
                kind: Kind::Website,
                name: NAME.to_string(),
                url: format!(url_format!(), "/"),
                ..ThingJsonLd::default()
            },
            potential_action: SearchActionJsonLd::new(),
        }
    }
}

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
            kind: Kind::AboutPage,
            name: format!("About {}", NAME),
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
                kind: Kind::Book,
                name: b.label.to_owned(),
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
            .enumerate()
            .map(|(i, c)| PartJsonLd {
                id: format!(url_format!(), c),
                kind: Kind::Chapter,
                name: format!("{} {}", book.name, i + 1),
            })
            .collect();
        let is_part_of = PartJsonLd {
            id: format!(url_format!(), links.books.url),
            kind: Kind::BookSeries,
            name: NAME.to_string(),
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
            kind: Kind::Book,
            name: links.book.label.to_owned(),
        };

        Self {
            is_part_of,
            position: reference.chapter,
            thing,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultsPageJsonLd {
    #[serde(flatten)]
    thing: ThingJsonLd,
}

impl SearchResultsPageJsonLd {
    pub(super) fn new(query: &str, url: &str) -> Self {
        Self {
            thing: ThingJsonLd {
                id: format!(url_format!(), url),
                kind: Kind::SearchResultsPage,
                name: format!("Search results for '{}'", query),
                url: format!(url_format!(), url),
                ..ThingJsonLd::default()
            },
        }
    }
}
