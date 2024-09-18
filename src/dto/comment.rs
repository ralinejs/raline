use serde::Deserialize;
use validator::Validate;

use crate::model::sea_orm_active_enums::CommentStatus;

#[derive(Debug, Validate, Deserialize)]
pub struct CommentQueryReq {
    pub r#type: ShowType,
    pub status: CommentStatus,
    pub owner: Owner,
    #[validate(length(max = 32, message = "查询关键字过长"))]
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum ShowType {
    List,
    Tree,
}

#[derive(Debug, Deserialize)]
pub enum Owner {
    All,
    Mine,
}
