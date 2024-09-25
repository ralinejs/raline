use sea_orm::DeriveColumn;
use sea_orm::EnumIter;
use serde::Deserialize;
use serde::Serialize;
use serde_with::formats::CommaSeparator;
use serde_with::serde_as;
use serde_with::StringWithSeparator;

use crate::model::view_counter;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct ViewCountQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub path: Vec<String>,
    #[serde(rename = "type")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, ColumnQueryAs>")]
    pub types: Vec<ColumnQueryAs>,
}

#[derive(Debug, Deserialize)]
pub struct SetViewCount {
    pub path: String,
    #[serde(default)]
    pub action: SetCountAction,
    pub r#type: ColumnQueryAs,
}

#[derive(Debug, Default, Deserialize)]
pub enum SetCountAction {
    #[default]
    #[serde(rename = "inc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(
    PartialEq, Eq, Hash, Copy, Clone, Debug, EnumIter, DeriveColumn, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ColumnQueryAs {
    #[serde(rename = "time")]
    Times,
    Reaction0,
    Reaction1,
    Reaction2,
    Reaction3,
    Reaction4,
    Reaction5,
    Reaction6,
    Reaction7,
    Reaction8,
}

impl ColumnQueryAs {
    pub fn get(&self, model: &view_counter::Model) -> i32 {
        match self {
            Self::Times => model.times,
            Self::Reaction0 => model.reaction0,
            Self::Reaction1 => model.reaction1,
            Self::Reaction2 => model.reaction2,
            Self::Reaction3 => model.reaction3,
            Self::Reaction4 => model.reaction4,
            Self::Reaction5 => model.reaction5,
            Self::Reaction6 => model.reaction6,
            Self::Reaction7 => model.reaction7,
            Self::Reaction8 => model.reaction8,
        }
    }
}
