//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.4

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub perm_id: i64,
    #[sea_orm(unique)]
    pub path: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::perm_container_perm_rel::Entity")]
    PermContainerPermRel,
}

impl Related<super::perm_container_perm_rel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PermContainerPermRel.def()
    }
}

impl Related<super::perm_container::Entity> for Entity {
    fn to() -> RelationDef {
        super::perm_container_perm_rel::Relation::PermContainer.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::perm_container_perm_rel::Relation::Permission
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
