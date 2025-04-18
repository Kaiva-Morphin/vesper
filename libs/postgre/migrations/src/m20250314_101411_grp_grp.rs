use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250311_141723_groups::Group;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
        .create_table(
            Table::create()
                .table(GroupGroup::Table)
                .if_not_exists()
                .col(integer(GroupGroup::ParentGroup))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-gg-parent")
                        .from(GroupGroup::Table, GroupGroup::ParentGroup)
                        .to(Group::Table, Group::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .col(integer(GroupGroup::Group))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-gg-grp")
                        .from(GroupGroup::Table, GroupGroup::Group)
                        .to(Group::Table, Group::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .primary_key(
                    Index::create()
                        .col(GroupGroup::ParentGroup)
                        .col(GroupGroup::Group)
                )
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupGroup::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GroupGroup {
    Table,
    ParentGroup,
    Group
}