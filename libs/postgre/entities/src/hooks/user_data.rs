use sea_orm::entity::prelude::*;

use crate::user_data::ActiveModel;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // if !insert { // todo! move to user service
        //     if !self.email.is_unchanged() {
        //         //todo: send email
        //     }
        //     if !self.login.is_unchanged() {
        //         //todo: send email
        //     }
        //     if !self.password.is_unchanged() {
        //         //todo: send email
        //     }
        // }
        if self.is_changed() {
            self.updated_at = Default::default();
        }
        if !self.login.is_unchanged() {
            self.last_login_change = Default::default();
        }
        Ok(self)
    }
}