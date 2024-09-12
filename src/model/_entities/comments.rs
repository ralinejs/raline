//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use super::sea_orm_active_enums::CommentStatus;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
    #[sea_orm(column_type = "custom(\"inet\")")]
    pub ip: String,
    pub link: Option<String>,
    pub mail: Option<String>,
    pub nick: Option<String>,
    pub pid: Option<i64>,
    pub rid: Option<i64>,
    pub sticky: bool,
    pub status: CommentStatus,
    pub star: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub ua: Option<String>,
    pub url: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
