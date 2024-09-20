use serde::Deserialize;
use super::Urls;

#[derive(Debug, Deserialize)]
pub struct ViewCountQuery {
    pub path: Option<Urls>,
}

#[derive(Debug, Deserialize)]
pub struct SetViewCount {
    pub path: String,
    pub action: SetCountAction,
}

#[derive(Debug, Default, Deserialize)]
pub enum SetCountAction {
    #[default]
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}
