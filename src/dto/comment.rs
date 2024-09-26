use crate::config::comrak::ComrakConfig;
use crate::config::RalineConfig;
use crate::model::comments;
use crate::model::comments::Model as Comments;
use crate::model::sea_orm_active_enums::CommentStatus;
use crate::model::sea_orm_active_enums::UserType;
use crate::model::users;
use crate::utils::jwt::OptionalClaims;
use comrak::markdown_to_html;
use derive_more::derive::From;
use sea_orm::prelude::DateTime;
use sea_orm::Order;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_with::formats::CommaSeparator;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use serde_with::StringWithSeparator;
use uaparser::Client;
use uaparser::UserAgent;
use uaparser::OS;
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
    #[validate(range(max = 200, message = "查询数据过多"))]
    #[serde_as(as = "DisplayFromStr")]
    pub limit: u64,
    #[serde(rename = "sortBy")]
    pub sort_by: OrderBy,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub offset: i64,
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

#[derive(Debug, Deserialize)]
pub enum OrderBy {
    #[serde(rename = "like_desc")]
    Like,
    #[serde(rename = "insertedAt_asc")]
    CreatedAtAsc,
    #[serde(rename = "insertedAt_desc")]
    CreatedAtDesc,
}

impl OrderBy {
    pub fn into_column_order(&self) -> (crate::model::comments::Column, Order) {
        match self {
            Self::Like => (crate::model::comments::Column::Star, Order::Desc),
            Self::CreatedAtAsc => (crate::model::comments::Column::CreatedAt, Order::Asc),
            Self::CreatedAtDesc => (crate::model::comments::Column::CreatedAt, Order::Desc),
        }
    }
}

#[derive(Debug, Serialize, From)]
#[serde(untagged)]
pub enum CommentQueryResp {
    Admin(AdminListResp),
    List(ListResp),
    Count { data: Vec<i64> },
    Recent(Vec<CommentResp>),
}

#[derive(Debug, Serialize)]
pub struct ListResp {
    pub count: u64,
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
    pub fn into_active_model(self) -> comments::ActiveModel {
        comments::ActiveModel {
            content: Set(self.comment),
            nick: Set(self.nick),
            ua: Set(self.ua),
            url: Set(self.url),
            link: Set(self.link),
            mail: Set(self.mail),
            pid: Set(self.pid),
            rid: Set(self.rid),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentResp {
    pub url: String,
    #[serde(with = "CommentStatusRef")]
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
impl CommentResp {
    pub async fn format(
        c: &Comments,
        users: &Vec<users::Model>,
        config: &RalineConfig,
        comrak: &ComrakConfig,
        login_user: &OptionalClaims,
    ) -> Self {
        let RalineConfig {
            disable_user_agent,
            disable_region,
            ..
        } = config;
        let client: Option<Client> = if *disable_user_agent { None } else { None };
        let is_admin = match &**login_user {
            None => false,
            Some(u) => u.ty == UserType::Admin,
        };
        let addr = if is_admin || !disable_region {
            None
        } else {
            None
        };
        let comment_html = markdown_to_html(&c.content, &comrak.into());
        let orig = if login_user.is_none() {
            None
        } else {
            Some(c.content.to_owned())
        };
        let user = users.iter().find(|u| c.user_id == Some(u.id));
        Self {
            url: c.url.to_owned(),
            status: c.status.to_owned(),
            comment: comment_html,
            inserted_at: c.created_at,
            link: c.link.to_owned(),
            nick: user.map(|u| u.username.clone()).or(c.nick.to_owned()),
            mail: user.and_then(|u| u.email.clone()).or(c.mail.to_owned()),
            r#type: user.map(|u| u.r#type.clone()),
            avatar: user.and_then(|u| u.avatar.clone()).unwrap_or_default(),
            pid: c.pid,
            rid: c.rid,
            user_id: c.user_id,
            sticky: c.sticky,
            like: c.star,
            object_id: c.id,
            level: 0,
            browser: client
                .clone()
                .map(|c| c.user_agent.to_string())
                .unwrap_or_default(),
            os: client.map(|c| c.os.to_string()).unwrap_or_default(),
            orig,
            addr,
            time: c.created_at.timestamp_micros(),
            children: Default::default(),
        }
    }
}

trait ToStringExt {
    fn to_string(&self) -> String;
}

impl<'a> ToStringExt for UserAgent<'a> {
    fn to_string(&self) -> String {
        let Self {
            family,
            major,
            minor,
            patch,
        } = self;
        let mut string = format!("{family}");
        if let Some(major) = major {
            string = string + " " + major;
        }
        if let Some(minor) = minor {
            string = string + "." + minor;
        }
        if let Some(patch) = patch {
            string = string + "." + patch;
        }
        string
    }
}
impl<'a> ToStringExt for OS<'a> {
    fn to_string(&self) -> String {
        let Self {
            family,
            major,
            minor,
            patch,
            ..
        } = self;
        let mut string = format!("{family}");
        if let Some(major) = major {
            string = string + " " + major;
        }
        if let Some(minor) = minor {
            string = string + "." + minor;
        }
        if let Some(patch) = patch {
            string = string + "." + patch;
        }
        string
    }
}

#[derive(Debug, Deserialize)]
pub struct CommentUpdateReq {
    pub comment: Option<String>,
    pub link: Option<String>,
    pub mail: Option<String>,
    pub nick: Option<String>,
    pub sticky: Option<bool>,
    pub status: Option<CommentStatus>,
    pub like: Option<bool>,
}

impl CommentUpdateReq {
    pub fn to_active_model(self, ty: UserType) -> comments::ActiveModel {
        comments::ActiveModel {
            ..Default::default()
        }
    }
}
