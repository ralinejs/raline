use serde::Deserialize;
use serde_with::formats::CommaSeparator;
use serde_with::serde_as;
use serde_with::StringWithSeparator;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct ViewCountQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub path: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetViewCount {
    pub path: String,
    pub action: SetCountAction,
    pub r#type: ColumnType,
}

#[derive(Debug, Default, Deserialize)]
pub enum SetCountAction {
    #[default]
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Debug, Deserialize)]
enum ColumnType {
    #[serde(rename = "time")]
    Time,
}
