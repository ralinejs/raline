use std::{cmp::max, collections::HashMap};

pub use super::_entities::page_view_counter::*;
use crate::views::pv_counter::{ColumnQueryAs, SetCountAction, SetViewCount};
use itertools::Itertools;
use sea_orm::{
    sqlx::types::chrono::Local, ActiveModelBehavior, ActiveModelTrait, ColumnTrait,
    ConnectionTrait, DbConn, DbErr, DerivePartialModel, EntityTrait, FromQueryResult, QueryFilter,
    Set,
};
use spring::async_trait;

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            self.created_at = Set(Local::now().naive_local());
        }
        self.updated_at = Set(Local::now().naive_local());
        Ok(self)
    }
}

impl Entity {
    pub async fn find_id_by_path<C, S>(db: &C, path: S) -> Result<Option<PathId>, DbErr>
    where
        C: ConnectionTrait,
        S: Into<String>,
    {
        let path_id = Entity::find()
            .filter(Column::Path.eq(path.into()))
            .into_partial_model::<PathId>()
            .one(db)
            .await?;
        Ok(path_id)
    }

    pub async fn find_ids_by_paths<C, V, S>(
        db: &C,
        paths: &V,
    ) -> Result<HashMap<String, i32>, DbErr>
    where
        C: ConnectionTrait,
        for<'a> &'a V: IntoIterator<Item = &'a S>,
        S: ToString,
    {
        let paths: Vec<String> = paths.into_iter().map(|s| s.to_string()).collect_vec();
        let path_ids = Entity::find()
            .filter(Column::Path.is_in(paths))
            .into_partial_model::<PathId>()
            .all(db)
            .await?;
        Ok(path_ids.into_iter().map(|p| (p.path, p.id)).collect())
    }

    pub async fn increase_by_path(db: &DbConn, q: &SetViewCount) -> Result<Model, DbErr> {
        let model = Entity::find()
            .filter(Column::Path.eq(&q.path))
            .one(db)
            .await?;

        let model = match model {
            None => {
                let mut am = ActiveModel {
                    path: Set(q.path.clone()),
                    ..Default::default()
                };
                match q.r#type {
                    ColumnQueryAs::Times => am.times = Set(1),
                    ColumnQueryAs::Reaction0 => am.reaction0 = Set(1),
                    ColumnQueryAs::Reaction1 => am.reaction1 = Set(1),
                    ColumnQueryAs::Reaction2 => am.reaction2 = Set(1),
                    ColumnQueryAs::Reaction3 => am.reaction3 = Set(1),
                    ColumnQueryAs::Reaction4 => am.reaction4 = Set(1),
                    ColumnQueryAs::Reaction5 => am.reaction5 = Set(1),
                    ColumnQueryAs::Reaction6 => am.reaction6 = Set(1),
                    ColumnQueryAs::Reaction7 => am.reaction7 = Set(1),
                    ColumnQueryAs::Reaction8 => am.reaction8 = Set(1),
                };
                am.insert(db).await?
            }
            Some(m) => {
                let mut am = ActiveModel {
                    id: Set(m.id),
                    ..Default::default()
                };
                let delta = match q.action {
                    SetCountAction::Asc => 1,
                    SetCountAction::Desc => -1,
                };
                match q.r#type {
                    ColumnQueryAs::Times => am.times = Set(max(m.times + delta, 0)),
                    ColumnQueryAs::Reaction0 => am.reaction0 = Set(max(m.reaction0 + delta, 0)),
                    ColumnQueryAs::Reaction1 => am.reaction1 = Set(max(m.reaction1 + delta, 0)),
                    ColumnQueryAs::Reaction2 => am.reaction2 = Set(max(m.reaction2 + delta, 0)),
                    ColumnQueryAs::Reaction3 => am.reaction3 = Set(max(m.reaction3 + delta, 0)),
                    ColumnQueryAs::Reaction4 => am.reaction4 = Set(max(m.reaction4 + delta, 0)),
                    ColumnQueryAs::Reaction5 => am.reaction5 = Set(max(m.reaction5 + delta, 0)),
                    ColumnQueryAs::Reaction6 => am.reaction6 = Set(max(m.reaction6 + delta, 0)),
                    ColumnQueryAs::Reaction7 => am.reaction7 = Set(max(m.reaction7 + delta, 0)),
                    ColumnQueryAs::Reaction8 => am.reaction8 = Set(max(m.reaction8 + delta, 0)),
                };
                am.update(db).await?
            }
        };

        Ok(model)
    }
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
pub struct PathId {
    #[sea_orm(from_col = "id")]
    pub id: i32,
    #[sea_orm(from_col = "path")]
    pub path: String,
}
