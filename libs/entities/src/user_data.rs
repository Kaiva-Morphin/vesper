//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.4

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_data")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    #[sea_orm(unique)]
    pub login: String,
    pub nickname: String,
    pub password: String,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique)]
    pub discord_id: Option<String>,
    #[sea_orm(unique)]
    pub google_id: Option<String>,
    pub last_login_change: Option<DateTime>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
    pub warn_suspicious_refresh: bool,
    pub allow_suspicious_refresh: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
