use sea_orm::sea_query::IntoCondition;
use serde::Deserialize;

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
        todo!()
    }
}
