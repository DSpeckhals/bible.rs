use serde_json;

use models::{Book, Verse};
use reference::Reference;
use ReceptusError;

#[derive(Serialize)]
struct ErrorPayload {
    message: String,
}

impl ErrorPayload {
    pub fn new(e: &ReceptusError) -> Self {
        Self {
            message: e.to_string(),
        }
    }

    pub fn json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

#[derive(Serialize)]
pub struct VersesPayload {
    book: Book,
    reference: String,
    verses: Vec<Verse>,
}

impl VersesPayload {
    pub fn new((book, verses): (Book, Vec<Verse>), reference: &Reference) -> Self {
        Self {
            book,
            reference: reference.to_string(),
            verses,
        }
    }
}

pub mod api;
pub mod view;
