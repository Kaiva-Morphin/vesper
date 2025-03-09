use sea_orm::{entity::prelude::*, sqlx::types::chrono::Utc, ActiveValue::Set};
use tracing::info;

use crate::user_data::ActiveModel;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.is_changed() {
            let t = Utc::now().naive_utc();
            self.updated_at = Set(t);
            if !self.login.is_unchanged() {
                self.last_login_change = Set(Some(t));
            }
        }
        Ok(self)
    }
}