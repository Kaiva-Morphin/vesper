use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20250306_130625_init::UserData, m20250311_141723_groups::Group};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        integer(Permission::Id)
                        .auto_increment()
                        // .unsigned() <-- doesn't support auto incr
                        .not_null()
                        .primary_key()
                        .unique_key()
                    )
                    .col(
                        string(Permission::Name)
                        .unique_key()
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(PermissionGroup::Table)
                    .if_not_exists()
                    .col(boolean(PermissionGroup::Value).not_null())
                    .col(
                        integer(PermissionGroup::Permission)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pg-per")
                            .from(PermissionGroup::Table, PermissionGroup::Permission)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(
                        integer(PermissionGroup::Group)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pg-grp")
                            .from(PermissionGroup::Table, PermissionGroup::Group)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)

                    )
                    .primary_key(
                        Index::create()
                            .col(PermissionGroup::Permission)
                            .col(PermissionGroup::Group)
                    )
                    .to_owned(),
            )
            .await?;
            manager
            .create_table(
                Table::create()
                    .table(PermissionUser::Table)
                    .if_not_exists()
                    .col(boolean(PermissionUser::Value).not_null())
                    .col(
                        integer(PermissionUser::Permission)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pu-per")
                            .from(PermissionUser::Table, PermissionUser::Permission)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(
                        uuid(PermissionUser::User)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pu-usr")
                            .from(PermissionUser::Table, PermissionUser::User)
                            .to(UserData::Table, UserData::UUID)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .primary_key(
                        Index::create()
                            .col(PermissionUser::Permission)
                            .col(PermissionUser::User)
                    )
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PermissionGroup::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PermissionUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
    Name
}

#[derive(DeriveIden)]
enum PermissionGroup {
    Table,
    Permission,
    Group,
    Value
}

#[derive(DeriveIden)]
enum PermissionUser {
    Table,
    Permission,
    User,
    Value
}