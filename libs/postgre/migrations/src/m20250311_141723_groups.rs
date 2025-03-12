use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250306_130625_init::UserData;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Group::Table)
                    .if_not_exists()
                    .col(integer(Group::Id)
                        .auto_increment()
                        // .unsigned() <-- doesn't support auto incr
                        .not_null()
                        .primary_key()
                        .unique_key()
                    )
                    .col(string_null(Group::Name))
                    .col(string_len_null(Group::Color, 7))
                    .col(boolean(Group::VisibleOnlyIn).not_null().default(false))
                    .col(boolean(Group::VisibleInSearch).not_null().default(false))
                    .col(boolean(Group::VisibleInProfile).not_null().default(false))
                    .col(string_null(Group::PinLink))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(UserGroup::Table)
                    .col(uuid(UserGroup::UserId).not_null())
                    .col(integer(UserGroup::GroupId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ug-gid")
                            .from(UserGroup::Table, UserGroup::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ug-uid")
                            .from(UserGroup::Table, UserGroup::UserId)
                            .to(UserData::Table, UserData::UUID)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(UserGroup::UserId)
                            .col(UserGroup::GroupId),
                    )
                    .to_owned()
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserGroup::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Group::Table).to_owned())
            .await
    }
}


#[derive(Iden)]
pub enum Group {
    Table,
    Id,
    Name,
    Color,
    VisibleOnlyIn,
    VisibleInSearch,
    VisibleInProfile,
    PinLink,
}

#[derive(Iden)]
pub enum UserGroup {
    Table,
    UserId,
    GroupId,
}