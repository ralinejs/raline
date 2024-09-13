use sea_orm::{sea_query::IntoCondition, ColumnTrait};
use serde::Deserialize;

use crate::model::view_counter;

#[derive(Debug, Deserialize)]
pub struct ViewCountQuery {
    pub path: Option<Path>,
}

#[derive(Debug, Deserialize)]
pub enum Path {
    Single(String),
    List(Vec<String>),
}

impl IntoCondition for Path {
    fn into_condition(self) -> sea_orm::Condition {
        match self {
            Self::Single(path) => view_counter::Column::Url.eq(path).into_condition(),
            Self::List(paths) => view_counter::Column::Url.is_in(paths).into_condition(),
        }
    }
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
