use sea_orm::{entity::prelude::*, sqlx::types::chrono::Utc, ActiveValue::Set};

use crate::user_data::ActiveModel;

//TODO: MOVE TO SQL.
#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.is_changed() {
            let t = Utc::now().naive_utc();
            self.updated_at = Set(t);
            if !self.uid.is_unchanged() {
                self.last_uid_change = Set(Some(t));
            }
        }
        Ok(self)
    }
}