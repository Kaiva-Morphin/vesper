use crate::AppState;
use sea_orm::*;


use anyhow::Result;



use postgre_entities::user_data;

impl AppState {
    pub async fn is_username_available(&self, username: String) -> Result<bool> {
        let v = user_data::Entity::find()
            .filter(user_data::Column::Username.eq(username))
            .one(&self.db).await;
        Ok(v?.is_none())
    }
}