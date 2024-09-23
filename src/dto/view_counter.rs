use serde::Deserialize;

use crate::model::view_counter;

#[derive(Debug, Deserialize)]
pub struct ViewCountQuery {
    pub path: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetViewCount {
    pub path: String,
    pub action: SetCountAction,
    // pub r#type: view_counter::Column,
}

#[derive(Debug, Default, Deserialize)]
pub enum SetCountAction {
    #[default]
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

