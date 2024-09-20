pub use super::_entities::comments::*;

use sea_orm::{sqlx::types::chrono::Local, ActiveModelBehavior, ConnectionTrait, DbErr, Set};
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