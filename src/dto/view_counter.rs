use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ViewCountQuery {
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
