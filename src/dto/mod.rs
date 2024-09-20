use serde::Deserialize;

pub mod user;
pub mod view_counter;
pub mod comment;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Urls {
    Single(String),
    List(Vec<String>),
}