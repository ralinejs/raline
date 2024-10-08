use crate::model::comments;
use crate::model::sea_orm_active_enums::CommentStatus;
use crate::model::sea_orm_active_enums::UserType;
use derive_more::derive::From;
use sea_orm::prelude::DateTime;
use sea_orm::Order;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_with::formats::CommaSeparator;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::BoolFromInt;
use serde_with::DisplayFromStr;
use serde_with::StringWithSeparator;
use validator::Validate;

#[serde_as]
#[derive(Debug, Validate, Deserialize)]
pub struct AdminCommentQuery {
    #[serde_as(as = "DisplayFromStr")]
    pub page: u64,
    #[validate(range(max = 200, message = "查询数据过多"))]
    #[serde(default = "default_size")]
    pub size: u64,
    pub status: CommentStatus,
    pub owner: Owner,
    #[validate(length(max = 32, message = "查询关键字过长"))]
    pub keyword: Option<String>,
}

fn default_size() -> u64 {
    20
}

#[serde_as]
#[derive(Debug, Validate, Deserialize)]
pub struct CountCommentQuery {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub url: Vec<String>,
}

#[serde_as]
#[derive(Debug, Validate, Deserialize)]
pub struct ListCommentQuery {
    pub path: String,
    #[serde(default = "comments::root_comment_id")]
    pub rid: i32,
    #[validate(range(min = 1, max = 200, message = "查询数据过多"))]
    #[serde_as(as = "DisplayFromStr")]
    pub limit: u64,
    #[serde(flatten)]
    pub sort_by: OrderBy,
}

/// 写在这里面方便后期数据量过大时，将limit pagination改造为keyset pagination
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "sortBy")]
pub enum OrderBy {
    #[serde(rename = "like_desc")]
    Like {
        #[serde(default)]
        #[serde_as(as = "DisplayFromStr")]
        offset: u64,
    },
    #[serde(rename = "insertedAt_asc")]
    CreatedAtAsc {
        #[serde(default)]
        #[serde_as(as = "DisplayFromStr")]
        offset: u64,
    },
    #[serde(rename = "insertedAt_desc")]
    CreatedAtDesc {
        #[serde(default)]
        #[serde_as(as = "DisplayFromStr")]
        offset: u64,
    },
}

impl OrderBy {
    pub fn into_column_order(self) -> ((crate::model::comments::Column, Order), u64) {
        match self {
            Self::Like { offset } => ((crate::model::comments::Column::Star, Order::Desc), offset),
            Self::CreatedAtAsc { offset } => (
                (crate::model::comments::Column::CreatedAt, Order::Asc),
                offset,
            ),
            Self::CreatedAtDesc { offset } => (
                (crate::model::comments::Column::CreatedAt, Order::Desc),
                offset,
            ),
        }
    }
}

#[serde_as]
#[derive(Debug, Validate, Deserialize)]
pub struct RecentCommentQuery {
    #[validate(range(max = 100, message = "查询数据过多"))]
    #[serde_as(as = "DisplayFromStr")]
    pub count: u64,
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

#[derive(Debug, Serialize, From)]
#[serde(untagged)]
pub enum CommentQueryResp {
    Admin(AdminListResp),
    List(ListResp),
    Count { data: Vec<i64> },
    Recent(Vec<CommentResp>),
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResp {
    pub count: u64,
    pub total_pages: u64,
    pub data: Vec<CommentResp>,
}

#[derive(Debug, Serialize)]
pub struct AdminListResp {
    pub page: u64,
    pub total_pages: u64,
    pub page_size: u64,
    pub spam_count: u64,
    pub waiting_count: u64,
    pub data: Vec<CommentResp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCommentReq {
    pub comment: String,
    pub ua: String,
    pub url: String,
    pub at: Option<String>,
    pub nick: Option<String>,
    pub link: Option<String>,
    pub mail: Option<String>,
    pub pid: Option<i32>,
    pub rid: Option<i32>,
}

impl AddCommentReq {
    pub fn into_active_model(self, page_id: i32) -> comments::ActiveModel {
        comments::ActiveModel {
            page_id: Set(page_id),
            content: Set(self.comment),
            nick: Set(self.nick),
            ua: Set(self.ua),
            link: Set(self.link),
            mail: Set(self.mail),
            pid: Set(self.pid.unwrap_or(0)),
            rid: Set(self.rid.unwrap_or(0)),
            ..Default::default()
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentResp {
    pub status: CommentStatus,
    pub comment: String,
    pub inserted_at: DateTime,
    pub link: Option<String>,
    pub nick: Option<String>,
    pub mail: Option<String>,
    pub pid: Option<i32>,
    pub rid: Option<i32>,
    #[serde(rename = "user_id")]
    pub user_id: Option<i32>,
    pub r#type: Option<UserType>,
    pub avatar: String,
    pub sticky: bool,
    pub like: i32,
    pub object_id: i32,
    pub level: i32,
    pub browser: String,
    pub os: String,
    pub orig: Option<String>,
    pub addr: Option<String>,
    pub time: i64,
    pub children: Vec<CommentResp>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct CommentUpdateReq {
    pub comment: Option<String>,
    pub link: Option<String>,
    pub mail: Option<String>,
    pub nick: Option<String>,
    #[serde_as(as = "Option<BoolFromInt>")]
    pub sticky: Option<bool>,
    pub status: Option<CommentStatus>,
    pub like: Option<bool>,
}

impl CommentUpdateReq {
    pub fn update_active_model(
        self,
        mut ac: comments::ActiveModel,
        ty: UserType,
    ) -> comments::ActiveModel {
        match ty {
            UserType::Normal => {
                if let Some(content) = self.comment {
                    ac.content = Set(content);
                }
            }
            UserType::Admin => {
                if let Some(content) = self.comment {
                    ac.content = Set(content);
                }
                if let Some(link) = self.link {
                    ac.link = Set(Some(link));
                }
                if let Some(mail) = self.mail {
                    ac.mail = Set(Some(mail));
                }
                if let Some(nick) = self.nick {
                    ac.nick = Set(Some(nick));
                }
                if let Some(sticky) = self.sticky {
                    ac.sticky = Set(sticky);
                }
                if let Some(status) = self.status {
                    ac.status = Set(status);
                }
            }
        };
        ac
    }
    pub fn is_empty(&self) -> bool {
        self.comment.is_none()
            && self.link.is_none()
            && self.mail.is_none()
            && self.nick.is_none()
            && self.sticky.is_none()
            && self.status.is_none()
    }
}
