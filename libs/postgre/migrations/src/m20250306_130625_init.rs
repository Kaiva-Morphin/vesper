use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
       
        manager
            .create_table(
                Table::create()
                    .table(UserData::Table)
                    .if_not_exists()
                    .col(uuid(UserData::UUID).not_null().primary_key().unique_key())
                    .col(string_len(UserData::Username, 32).not_null().unique_key())
                    .col(string_len(UserData::Nickname, 32).not_null())
                    .col(string(UserData::Password).not_null())
                    .col(string(UserData::Email).not_null().unique_key())
                    .col(string_null(UserData::DiscordId).unique_key())
                    .col(string_null(UserData::GoogleId).unique_key())
                    .col(timestamp(UserData::Created))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserData {
    Table,
    UUID,
    Username,
    Nickname,
    Password,
    Email,
    DiscordId,
    GoogleId,
    Created
}
