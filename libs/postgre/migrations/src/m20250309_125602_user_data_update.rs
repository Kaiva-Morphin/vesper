use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(Table::alter()
                .table(UserData::Table)
                .add_column(boolean(UserData::WarnSuspiciousRefresh).not_null().default(true))
                .add_column(boolean(UserData::AllowSuspiciousRefresh).not_null().default(false))
            .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(Table::alter()
                .table(UserData::Table)
                .drop_column(UserData::WarnSuspiciousRefresh)
                .drop_column(UserData::AllowSuspiciousRefresh)
            .to_owned()
        ).await
    }
}

#[derive(DeriveIden)]
enum UserData {
    Table,
    UUID,
    Login,
    Nickname,
    Password,
    Email,
    DiscordId,
    GoogleId,
    LastLoginChange,
    UpdatedAt,
    CreatedAt,
    WarnSuspiciousRefresh,
    AllowSuspiciousRefresh
}
