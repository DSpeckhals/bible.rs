use serde_derive::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct SearchParams {
    q: String,
}

pub mod api;
pub mod view;
