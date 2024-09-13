pub use super::_entities::view_counter::*;

use sea_orm::{
    sqlx::types::chrono::Local, ActiveModelBehavior, ActiveModelTrait, ColumnTrait,
    ConnectionTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set,
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
    pub async fn increase_by_path<S: Into<String>>(db: &DbConn, path: S) -> Result<Model, DbErr> {
        let path = path.into();
        let model = Entity::find().filter(Column::Url.eq(&path)).one(db).await?;

        let model = match model {
            None => {
                ActiveModel {
                    url: Set(path),
                    ..Default::default()
                }
                .insert(db)
                .await?
            }
            Some(m) => {
                ActiveModel {
                    id: Set(m.id),
                    times: Set(m.times + 1),
                    ..Default::default()
                }
                .update(db)
                .await?
            }
        };

        Ok(model)
    }
}
