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
                    .table(AllowPermissionGroup::Table)
                    .if_not_exists()
                    .col(
                        integer(AllowPermissionGroup::Permission)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-allow-pg-per")
                            .from(AllowPermissionGroup::Table, AllowPermissionGroup::Permission)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)

                    )
                    .col(
                        integer(AllowPermissionGroup::Group)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-allow-pg-grp")
                            .from(AllowPermissionGroup::Table, AllowPermissionGroup::Group)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)

                    )
                    .primary_key(
                        Index::create()
                            .col(AllowPermissionGroup::Permission)
                            .col(AllowPermissionGroup::Group)
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(DenyPermissionGroup::Table)
                    .if_not_exists()
                    .col(
                        integer(DenyPermissionGroup::Permission)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-deny-pg-per")
                            .from(DenyPermissionGroup::Table, DenyPermissionGroup::Permission)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(
                        integer(DenyPermissionGroup::Group)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-deny-pg-grp")
                            .from(DenyPermissionGroup::Table, DenyPermissionGroup::Group)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .primary_key(
                        Index::create()
                            .col(DenyPermissionGroup::Permission)
                            .col(DenyPermissionGroup::Group)
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(AllowPermissionUser::Table)
                    .if_not_exists()
                    .col(
                        integer(AllowPermissionUser::Permission)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-allow-pu-per")
                            .from(AllowPermissionUser::Table, AllowPermissionUser::Permission)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(
                        uuid(AllowPermissionUser::User)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-allow-pu-usr")
                            .from(AllowPermissionUser::Table, AllowPermissionUser::User)
                            .to(UserData::Table, UserData::UUID)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .primary_key(
                        Index::create()
                            .col(AllowPermissionUser::Permission)
                            .col(AllowPermissionUser::User)
                    )
                    .to_owned(),
            ).await?;
            manager
            .create_table(
                Table::create()
                    .table(DenyPermissionUser::Table)
                    .if_not_exists()
                    .col(
                        integer(DenyPermissionUser::Permission)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-deny-pu-per")
                            .from(DenyPermissionUser::Table, DenyPermissionUser::Permission)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .col(
                        uuid(DenyPermissionUser::User)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-deny-pu-usr")
                            .from(DenyPermissionUser::Table, DenyPermissionUser::User)
                            .to(UserData::Table, UserData::UUID)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .primary_key(
                        Index::create()
                            .col(DenyPermissionUser::Permission)
                            .col(DenyPermissionUser::User)
                    )
                    .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AllowPermissionGroup::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DenyPermissionGroup::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AllowPermissionUser::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DenyPermissionUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum AllowPermissionGroup {
    Table,
    Permission,
    Group,
}

#[derive(DeriveIden)]
enum DenyPermissionGroup {
    Table,
    Permission,
    Group,
}

#[derive(DeriveIden)]
enum AllowPermissionUser {
    Table,
    Permission,
    User,
}

#[derive(DeriveIden)]
enum DenyPermissionUser {
    Table,
    Permission,
    User,
}