use migration::{OnConflict, SimpleExpr};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use anyhow::Result;

pub trait Lifetime {fn get_lifetime(&self) -> i64;}

pub trait Path {
    fn to_key(&self) -> String;
    fn construct_id(&self, id: u64) -> impl Id + Lifetime;
    fn value(&self) -> &str;
}

pub trait Id {
    fn to_key(&self) -> String;
    fn construct_path(&self, path: String) -> impl Path + Lifetime;
    fn value(&self) -> u64;
}
#[derive(PartialEq, Debug, Clone)]
pub struct Wildcard(pub String);
#[derive(PartialEq, Debug, Clone)]
pub struct WildcardId(pub u64);
#[derive(PartialEq, Debug, Clone)]
pub struct Perm(pub String);
#[derive(PartialEq, Debug, Clone)]
pub struct PermId(pub u64);

const PERM_ID_REL_PREFIX : &'static str = "PERM_ID";
const ID_PERM_REL_PREFIX : &'static str = "ID_PERM";
fn perm_to_key(perm: &str) -> String {
    format!("{}::{}", PERM_ID_REL_PREFIX, perm)
    
}

fn perm_id_to_key(id: &u64) -> String {
    format!("{}::{}", ID_PERM_REL_PREFIX, id)
}

const WILDCARD_ID_REL_PREFIX : &'static str = "WILDCARD_ID";
const ID_WILDCARD_REL_PREFIX : &'static str = "ID_WILDCARD";
fn wildcard_to_key(perm: &str) -> String {
    format!("{}::{}", WILDCARD_ID_REL_PREFIX, perm)
    
}

fn wildcard_id_to_key(id: &u64) -> String {
    format!("{}::{}", ID_WILDCARD_REL_PREFIX, id)
}


impl Path for Wildcard {
    fn to_key(&self) -> String {
        wildcard_to_key(&self.0)
    }
    fn construct_id(&self, id: u64) -> impl Id + Lifetime {
        WildcardId(id)
    }
    fn value(&self) -> &str {
        &self.0
    }
}
impl Id for WildcardId {
    fn to_key(&self) -> String {
        wildcard_id_to_key(&self.0)
    }
    fn construct_path(&self, path: String) -> impl Path + Lifetime {
        Wildcard(path)
    }
    fn value(&self) -> u64 {
        self.0
    }
}
impl Path for Perm {
    fn to_key(&self) -> String {
        perm_to_key(&self.0)
    }
    fn construct_id(&self, id: u64) -> impl Id + Lifetime {
        PermId(id)
    }
    fn value(&self) -> &str {
        &self.0
    }
}
impl Id for PermId {
    fn to_key(&self) -> String {
        perm_id_to_key(&self.0)
    }
    fn construct_path(&self, path: String) -> impl Path + Lifetime {
        Perm(path)
    }
    fn value(&self) -> u64 {
        self.0
    }
}

pub trait DbStructRel {
    type Entity: EntityTrait;
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity> + Send + Sync;
    type Column: ColumnTrait;
    fn equal_path(&self) -> SimpleExpr;
    fn path_col() -> Self::Column;
    fn path_in(things: Vec<Self>) -> SimpleExpr where Self : Sized;
    fn active_model(&self) -> Self::ActiveModel;
    fn construct_id(entity: <Self::Entity as EntityTrait>::Model) -> impl Id + Lifetime;
    fn construct_path(entity: <Self::Entity as EntityTrait>::Model) -> impl Path + Lifetime;
}

impl DbStructRel for Perm {
    type Entity = postgre_entities::permission::Entity;
    type ActiveModel = postgre_entities::permission::ActiveModel;
    type Column = postgre_entities::permission::Column;
    fn equal_path(&self) -> SimpleExpr {
        postgre_entities::permission::Column::Path.eq(self.value().to_string())
    }
    fn path_in(things: Vec<Self>) -> SimpleExpr where Self : Sized {
        postgre_entities::permission::Column::Path.is_in(&things)
    }
    fn path_col() -> Self::Column {
        Self::Column::Path
    }
    fn active_model(&self) -> Self::ActiveModel {
        Self::ActiveModel {
            path: Set(self.value().to_string()),
            ..Default::default()
        }
    }
    fn construct_id(entity: <Self::Entity as EntityTrait>::Model) -> impl Id + Lifetime {
        PermId(entity.perm_id as u64)
    }
    fn construct_path(entity: <Self::Entity as EntityTrait>::Model) -> impl Path + Lifetime {
        Perm(entity.path)
    }
}

impl DbStructRel for PermId {
    type Entity = postgre_entities::permission::Entity;
    type ActiveModel = postgre_entities::permission::ActiveModel;
    type Column = postgre_entities::permission::Column;
    fn equal_path(&self) -> SimpleExpr {
        postgre_entities::permission::Column::PermId.eq(self.value())
    }
    fn path_in(things: Vec<Self>) -> SimpleExpr where Self : Sized {
        postgre_entities::permission::Column::PermId.is_in(&things)
    }
    fn path_col() -> Self::Column {
        Self::Column::PermId
    }
    fn active_model(&self) -> Self::ActiveModel {
        Self::ActiveModel {
            perm_id: Set(self.value() as i64),
            ..Default::default()
        }
    }
    fn construct_id(entity: <Self::Entity as EntityTrait>::Model) -> impl Id + Lifetime {
        PermId(entity.perm_id as u64)
    }
    fn construct_path(entity: <Self::Entity as EntityTrait>::Model) -> impl Path + Lifetime {
        Perm(entity.path)
    }
}

impl DbStructRel for Wildcard {
    type Entity = postgre_entities::perm_wildcard::Entity;
    type ActiveModel = postgre_entities::perm_wildcard::ActiveModel;
    type Column = postgre_entities::perm_wildcard::Column;
    fn equal_path(&self) -> SimpleExpr {
        postgre_entities::perm_wildcard::Column::Path.eq(self.value().to_string())
    }
    fn path_in(things: Vec<Self>) -> SimpleExpr where Self : Sized {
        postgre_entities::perm_wildcard::Column::Path.is_in(&things)
    }
    fn path_col() -> Self::Column {
        Self::Column::Path
    }
    fn active_model(&self) -> Self::ActiveModel {
        Self::ActiveModel {
            path: Set(self.value().to_string()),
            ..Default::default()
        }
    }
    fn construct_id(entity: <Self::Entity as EntityTrait>::Model) -> impl Id + Lifetime {
        WildcardId(entity.perm_wildcard_id as u64)
    }
    fn construct_path(entity: <Self::Entity as EntityTrait>::Model) -> impl Path + Lifetime {
        Wildcard(entity.path)
    }
}

impl DbStructRel for WildcardId {
    type Entity = postgre_entities::perm_wildcard::Entity;
    type ActiveModel = postgre_entities::perm_wildcard::ActiveModel;
    type Column = postgre_entities::perm_wildcard::Column;
    fn equal_path(&self) -> SimpleExpr {
        postgre_entities::perm_wildcard::Column::PermWildcardId.eq(self.value())
    }
    fn path_in(things: Vec<Self>) -> SimpleExpr where Self : Sized {
        postgre_entities::perm_wildcard::Column::PermWildcardId.is_in(&things)
    }
    fn path_col() -> Self::Column {
        Self::Column::PermWildcardId
    }
    fn active_model(&self) -> Self::ActiveModel {
        Self::ActiveModel {
            perm_wildcard_id: Set(self.value() as i64),
            ..Default::default()
        }
    }
    fn construct_id(entity: <Self::Entity as EntityTrait>::Model) -> impl Id + Lifetime {
        PermId(entity.perm_wildcard_id as u64)
    }
    fn construct_path(entity: <Self::Entity as EntityTrait>::Model) -> impl Path + Lifetime {
        Perm(entity.path)
    }
}

pub trait DbInsert {
    fn insert(&self, db: &DatabaseConnection) -> impl std::future::Future<Output = Result<()>> + Send;
    fn insert_many(db: &DatabaseConnection, things: Vec<Self>) -> impl std::future::Future<Output = Result<()>> + Send where Self : Sized;
}
pub trait DbDelete {
    fn delete(&self, db: &DatabaseConnection) -> impl std::future::Future<Output = Result<()>> + Send;
    fn delete_many(db: &DatabaseConnection, things: Vec<Self>) -> impl std::future::Future<Output = Result<()>> + Send where Self: Sized;
}
pub trait DbGet {
    fn get_id_from_db(&self, db: &DatabaseConnection) -> impl std::future::Future<Output = Result<Option<impl Id + Lifetime>>> + Send;
    fn get_path_from_db(&self, db: &DatabaseConnection) -> impl std::future::Future<Output = Result<Option<impl Path + Lifetime>>> + Send;
}

impl From<&Perm> for sea_orm::Value {
    fn from(v: &Perm) -> Self {
        sea_orm::Value::String(Some(Box::new(v.value().to_string())))
    }
}

impl From<&Wildcard> for sea_orm::Value {
    fn from(v: &Wildcard) -> Self {
        sea_orm::Value::String(Some(Box::new(v.value().to_string())))
    }
}

impl From<&PermId> for sea_orm::Value {
    fn from(v: &PermId) -> Self {
        sea_orm::Value::BigInt(Some(v.value() as i64))
    }
}

impl From<&WildcardId> for sea_orm::Value {
    fn from(v: &WildcardId) -> Self {
        sea_orm::Value::BigInt(Some(v.value() as i64))
    }
}

impl<T: DbStructRel + Sync + Send> DbDelete for T {
    async fn delete(&self, db: &DatabaseConnection) -> Result<()> {
        T::Entity::delete_many()
            .filter(self.equal_path()).exec(db).await?;
        Ok(())
    }
    async fn delete_many(db: &DatabaseConnection, things: Vec<Self>) -> Result<()> {
        T::Entity::delete_many()
            .filter(Self::path_in(things))
            .exec(db)
            .await?;
        Ok(())
    }
}

impl<T: DbStructRel + Sync + Send> DbInsert for T {
    async fn insert(&self, db: &DatabaseConnection) -> Result<()> {
        let m = self.active_model();
        T::Entity::insert(m).on_conflict(
            OnConflict::column(Self::path_col()).do_nothing().to_owned()
        ).exec(db).await?;
        Ok(())
    }
    async fn insert_many(db: &DatabaseConnection, things: Vec<Self>) -> Result<()> {
        let models : Vec<T::ActiveModel> = things.into_iter().map(|v| v.active_model()).collect();
        if models.is_empty() {return Ok(());}
        T::Entity::insert_many(models).on_conflict(
            OnConflict::column(postgre_entities::permission::Column::Path).do_nothing().to_owned()
        ).exec(db).await?;
        Ok(())
    }
}

impl<T: DbStructRel + Sync + Send> DbGet for T {
    async fn get_id_from_db(&self, db: &DatabaseConnection) -> Result<Option<impl Id + Lifetime>> {
        let p = T::Entity::find().filter(self.equal_path()).one(db).await?;
        Ok(p.and_then(|v| Some(T::construct_id(v))))
    }
    async fn get_path_from_db(&self, db: &DatabaseConnection) -> Result<Option<impl Path + Lifetime>> {
        let p = T::Entity::find().filter(self.equal_path()).one(db).await?;
        Ok(p.and_then(|v| Some(T::construct_path(v))))
    }
}
