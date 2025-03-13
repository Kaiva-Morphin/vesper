//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.4

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "permission_group")]
pub struct Model {
    pub value: bool,
    #[sea_orm(primary_key, auto_increment = false)]
    pub permission: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub group: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::group::Entity",
        from = "Column::Group",
        to = "super::group::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Group,
    #[sea_orm(
        belongs_to = "super::permission::Entity",
        from = "Column::Permission",
        to = "super::permission::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Permission,
}

impl Related<super::group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::permission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Permission.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
