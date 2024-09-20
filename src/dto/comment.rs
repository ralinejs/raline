use super::Urls;
use crate::model::sea_orm_active_enums::CommentStatus;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct AdminCommentQuery {
    pub page: u64,
    #[validate(range(max = 200, message = "查询数据过多"))]
    pub size: u64,
    pub status: CommentStatus,
    pub owner: Owner,
    #[validate(length(max = 32, message = "查询关键字过长"))]
    pub keyword: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CountCommentQuery {
    pub url: Urls,
}

#[derive(Debug, Validate, Deserialize)]
pub struct ListCommentQuery {
    pub path: String,
    pub order_by: OrderBy,
    #[validate(range(max = 200, message = "查询数据过多"))]
    pub limit: u16,
    pub offset: i64,
}

#[derive(Debug, Validate, Deserialize)]
pub struct RecentCommentQuery {
    #[validate(range(max = 100, message = "查询数据过多"))]
    pub count: u16,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CommentQueryReq {
    #[serde(rename = "admin")]
    Admin(AdminCommentQuery),
    #[serde(rename = "count")]
    Count(CountCommentQuery),
    #[serde(rename = "list")]
    List(ListCommentQuery),
    #[serde(rename = "recent")]
    Recent(RecentCommentQuery),
}

impl Validate for CommentQueryReq {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            Self::Admin(q) => q.validate(),
            Self::Count(q) => q.validate(),
            Self::List(q) => q.validate(),
            Self::Recent(q) => q.validate(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum Owner {
    All,
    Mine,
}

#[derive(Debug, Deserialize)]
pub enum OrderBy {
    Like,
    CreatedAt,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum CommentQueryResp {
    Count(CountResp),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum CountResp {
    Single(u64),
    List(Vec<u64>),
}

impl From<u64> for CountResp {
    fn from(value: u64) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<u64>> for CountResp {
    fn from(value: Vec<u64>) -> Self {
        Self::List(value)
    }
}

impl From<CountResp> for CommentQueryResp {
    fn from(value: CountResp) -> Self {
        Self::Count(value)
    }
}
