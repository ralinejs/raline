use super::Urls;
use crate::model::comments::Model as Comments;
use crate::model::sea_orm_active_enums::CommentStatus;
use derive_more::derive::From;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use validator::Validate;

#[serde_as]
#[derive(Debug, Validate, Deserialize)]
pub struct AdminCommentQuery {
    #[serde_as(as = "DisplayFromStr")]
    pub page: u64,
    #[validate(range(max = 200, message = "查询数据过多"))]
    #[serde(default = "default_size")]
    pub size: u64,
    #[serde(with = "CommentStatusRef")]
    pub status: CommentStatus,
    pub owner: Owner,
    #[validate(length(max = 32, message = "查询关键字过长"))]
    pub keyword: Option<String>,
}

fn default_size() -> u64 {
    20
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
    pub limit: u64,
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
    #[serde(rename = "all")]
    All,
    #[serde(rename = "mine")]
    Mine,
}

#[derive(Debug, Deserialize)]
pub enum OrderBy {
    #[serde(rename = "like_desc")]
    Like,
    #[serde(rename = "insert_at_desc")]
    CreatedAt,
}

impl OrderBy {
    pub fn into_column(&self) -> crate::model::comments::Column {
        match self {
            Self::Like => crate::model::comments::Column::Star,
            Self::CreatedAt => crate::model::comments::Column::CreatedAt,
        }
    }
}

#[derive(Debug, Serialize, From)]
#[serde(untagged)]
pub enum CommentQueryResp {
    Count(CountResp),
    List(ListResp),
    Admin(AdminListResp),
}

#[derive(Debug, Serialize, From)]
#[serde(untagged)]
pub enum CountResp {
    Single(u64),
    List(Vec<u64>),
}

#[derive(Debug, Serialize)]
pub struct ListResp {
    pub total: u64,
    pub data: Vec<Comments>,
}

#[derive(Debug, Serialize)]
pub struct AdminListResp {
    pub page: u64,
    pub total_pages: u64,
    pub page_size: u64,
    pub spam_count: u64,
    pub waiting_count: u64,
    pub data: Vec<Comments>,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "CommentStatus")]
pub enum CommentStatusRef {
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "deleted")]
    Deleted,
    #[serde(rename = "spam")]
    Spam,
    #[serde(rename = "waiting")]
    Waiting,
}
