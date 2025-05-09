use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PermContainer::Table)
                    .if_not_exists()
                    .col(big_integer(PermContainer::Id)
                        .auto_increment()
                        // .unsigned() <-- doesn't support auto incr
                        .not_null()
                        .primary_key()
                        .unique_key()
                    )
                    .col(integer(PermContainer::Weight).not_null())
                    .col(string_null(PermContainer::Name))
                    .col(string_len_null(PermContainer::Color, 7))
                    .col(boolean(PermContainer::VisibleInSearch).not_null().default(false))
                    .col(boolean(PermContainer::VisibleInProfile).not_null().default(false))
                    .col(string_null(PermContainer::PinLink))
                    .to_owned(),
            )
            .await?;
        
        manager
            .alter_table(Table::alter()
                .table(UserData::Table)
                .add_column(big_integer(UserData::PermContainer).not_null())
                .add_foreign_key(
                    ForeignKey::create()
                        .name("fk-user-perm_container")
                        .from(UserData::Table, UserData::PermContainer)
                        .to(PermContainer::Table, PermContainer::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .get_foreign_key()
                )
            .to_owned()
            ).await?;
        
        manager
            .create_table(
                Table::create()
                    .table(PermContainerContainerRel::Table)
                    .if_not_exists()
                    .col(boolean(PermContainerContainerRel::Value).not_null().default(true))
                    .col(big_integer(PermContainerContainerRel::PermContainerId).not_null())
                    .col(big_integer(PermContainerContainerRel::ChildPermContainerId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-perm_container_rel-child_perm_container")
                            .from(PermContainerContainerRel::Table, PermContainerContainerRel::ChildPermContainerId)
                            .to(PermContainer::Table, PermContainer::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-perm_container_rel-perm_container")
                            .from(PermContainerContainerRel::Table, PermContainerContainerRel::PermContainerId)
                            .to(PermContainer::Table, PermContainer::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .primary_key(Index::create().col(PermContainerContainerRel::PermContainerId).col(PermContainerContainerRel::ChildPermContainerId))
                    .to_owned()
            )
            .await?;
        
        
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(big_integer(Permission::PermId).auto_increment().not_null().primary_key().unique_key())
                    .col(string(Permission::Path).not_null().unique_key())
                    .to_owned()
            ).await?;
        
        manager
            .create_table(
                Table::create()
                    .table(PermContainerPermRel::Table)
                    .if_not_exists()
                    .col(boolean(PermContainerPermRel::Value).not_null().default(true))
                    .col(big_integer(PermContainerPermRel::PermContainerId).not_null())
                    .col(big_integer(PermContainerPermRel::PermId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-perm_container_perm_rel-perm_container")
                            .from(PermContainerPermRel::Table, PermContainerPermRel::PermContainerId)
                            .to(PermContainer::Table, PermContainer::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-perm_container_perm_rel-perm")
                            .from(PermContainerPermRel::Table, PermContainerPermRel::PermId)
                            .to(Permission::Table, Permission::PermId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .primary_key(Index::create().col(PermContainerPermRel::PermContainerId).col(PermContainerPermRel::PermId))
                    .to_owned()
            )
            .await?;
        
        manager
            .create_table(
                Table::create()
                    .table(PermWildcard::Table)
                    .if_not_exists()
                    .col(big_integer(PermWildcard::PermWildcardId).auto_increment().not_null().primary_key().unique_key())
                    .col(string(PermWildcard::Path).not_null().unique_key())
                    .to_owned()
            ).await?;
        
        manager
            .create_table(
                Table::create()
                    .table(PermContainerWildcardRel::Table)
                    .if_not_exists()
                    .col(boolean(PermContainerWildcardRel::Value).not_null().default(true))
                    .col(big_integer(PermContainerWildcardRel::PermContainerId).not_null())
                    .col(big_integer(PermWildcard::PermWildcardId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-perm_container_wildcard_rel-perm_container")
                            .from(PermContainerWildcardRel::Table, PermContainerWildcardRel::PermContainerId)
                            .to(PermContainer::Table, PermContainer::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-perm_container_wildcard_rel-perm")
                            .from(PermContainerWildcardRel::Table, PermContainerWildcardRel::PermWildcardId)
                            .to(PermWildcard::Table, PermWildcard::PermWildcardId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .primary_key(Index::create().col(PermContainerWildcardRel::PermContainerId).col(PermContainerWildcardRel::PermWildcardId))
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PermContainerWildcardRel::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PermWildcard::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PermContainerPermRel::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PermContainerContainerRel::Table).to_owned())
            .await?;

        manager
            .alter_table(Table::alter()
                .table(UserData::Table)
                .drop_column(UserData::PermContainer)
            .to_owned()
        ).await?;

        manager
            .drop_table(Table::drop().table(PermContainer::Table).to_owned())
            .await?;
        Ok(())
    }
}


#[derive(Iden)]
pub enum PermContainer {
    Table,
    Id,
    Name,
    Color,
    VisibleInSearch,
    VisibleInProfile,
    PinLink,
    Weight,
}

#[derive(DeriveIden)]
enum UserData {
    Table,
    PermContainer,
}


#[derive(Iden)]
pub enum PermContainerContainerRel {
    Table,
    PermContainerId,
    ChildPermContainerId,
    Value
}

#[derive(Iden)]
pub enum PermWildcard {
    Table,
    PermWildcardId,
    Path
}

#[derive(Iden)]
pub enum Permission {
    Table,
    PermId,
    Path
}

#[derive(Iden)]
pub enum PermContainerWildcardRel {
    Table,
    PermContainerId,
    PermWildcardId,
    Value
}

#[derive(Iden)]
pub enum PermContainerPermRel {
    Table,
    PermContainerId,
    PermId,
    Value
}