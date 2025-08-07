use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

use crate::m20250306_130625_init::{UserData as UserDataTable};

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(Table::alter()
                .table(UserData::Table)
                .add_column(string_null(UserData::Avatar).null())
                .add_column(array(UserData::Badges, ColumnType::SmallInteger).default(Vec::<i16>::new()))
            .to_owned()
        ).await?;

        manager
            .create_table(Table::create()
                .table(Post::Table)
                .if_not_exists()
                .col(uuid(Post::GUID).not_null().primary_key().unique_key())
                .col(uuid(Post::UserGUID).not_null().unique_key())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-post-owner_guid")
                        .from(Post::Table, Post::UserGUID)
                        .to(UserDataTable::Table, UserDataTable::GUID)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                .col(string_null(Post::Content))
                .col(array_null(Post::Attachments, ColumnType::String(StringLen::None)).null())
                .col(timestamp(Post::CreatedAt).extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                .col(timestamp(Post::UpdatedAt).extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
            .to_owned()
        ).await?;
        manager
            .create_table(Table::create()
                .table(UserProfile::Table)
                .if_not_exists()
                .col(uuid(UserProfile::UserGUID).not_null().unique_key().primary_key())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-user_profile-guid")
                        .from(UserProfile::Table, UserProfile::UserGUID)
                        .to(UserDataTable::Table, UserDataTable::GUID)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                .col(string_null(UserProfile::EncodedTheme).null())
                .col(string_null(UserProfile::Background).null())
                .col(string_null(UserProfile::Status).null())
            .to_owned()
        ).await?;
        manager
            .create_table(Table::create()
                .table(UserMiniProfile::Table)
                .if_not_exists()
                .col(uuid(UserMiniProfile::UserGUID).not_null().unique_key().primary_key())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-user_miniprofile-guid")
                        .from(UserMiniProfile::Table, UserMiniProfile::UserGUID)
                        .to(UserDataTable::Table, UserDataTable::GUID)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                .col(string_null(UserMiniProfile::EncodedTheme).null())
                .col(string_null(UserMiniProfile::Background).null())
                .col(string_null(UserMiniProfile::Status).null())
            .to_owned()
        ).await?;
        manager
            .create_table(Table::create()
                .table(Friends::Table)
                .if_not_exists()
                .col(uuid(Friends::MinUserGUID).not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-friends-guid_min")
                        .from(Friends::Table, Friends::MinUserGUID)
                        .to(UserDataTable::Table, UserDataTable::GUID)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                .col(uuid(Friends::MaxUserGUID).not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-friends-guid_max")
                        .from(Friends::Table, Friends::MaxUserGUID)
                        .to(UserDataTable::Table, UserDataTable::GUID)
                        .on_delete(ForeignKeyAction::Cascade)
                    )
                .col(boolean(Friends::MinUserGUIDDecision).not_null().default(false))
                .col(boolean(Friends::MaxUserGUIDDecision).not_null().default(false))
                .primary_key(Index::create().col(Friends::MinUserGUID).col(Friends::MaxUserGUID))
            .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(Table::alter()
                .table(UserData::Table)
                .drop_column(UserData::Avatar)
                .drop_column(UserData::Badges)
            .to_owned()
        ).await?;

        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(UserProfile::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserMiniProfile::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Friends::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    GUID,
    UserGUID,
    Content,
    Attachments,
    CreatedAt,
    UpdatedAt,
}


#[derive(DeriveIden)]
enum UserData {
    Table,
    Avatar,
    Badges,
}


#[derive(DeriveIden)]
enum UserProfile {
    Table,
    UserGUID,
    EncodedTheme,
    Background,
    Status,
}



#[derive(DeriveIden)]
enum UserMiniProfile {
    Table,
    UserGUID,
    EncodedTheme,
    Background,
    Status,
}



#[derive(DeriveIden)]
enum Friends {
    Table,
    MinUserGUID,
    MaxUserGUID,
    MinUserGUIDDecision,
    MaxUserGUIDDecision,
}