/* @generated and managed by dsync */

#[allow(unused)]
use crate::diesel::*;
use crate::schema::*;

pub type ConnectionType = diesel::pg::PgConnection;

/// Struct representing a row in table `user_data`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Queryable, diesel::Selectable, diesel::QueryableByName, diesel::Identifiable)]
#[diesel(table_name=user_data, primary_key(uuid))]
pub struct UserData {
    /// Field representing column `uuid`
    pub uuid: uuid::Uuid,
    /// Field representing column `username`
    pub username: String,
    /// Field representing column `nickname`
    pub nickname: String,
    /// Field representing column `password`
    pub password: String,
    /// Field representing column `email`
    pub email: String,
    /// Field representing column `discord_id`
    pub discord_id: Option<String>,
    /// Field representing column `google_id`
    pub google_id: Option<String>,
    /// Field representing column `created`
    pub created: i64,
}

/// Create Struct for a row in table `user_data` for [`UserData`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Insertable)]
#[diesel(table_name=user_data)]
pub struct CreateUserData {
    /// Field representing column `uuid`
    pub uuid: uuid::Uuid,
    /// Field representing column `username`
    pub username: String,
    /// Field representing column `nickname`
    pub nickname: String,
    /// Field representing column `password`
    pub password: String,
    /// Field representing column `email`
    pub email: String,
    /// Field representing column `discord_id`
    pub discord_id: Option<String>,
    /// Field representing column `google_id`
    pub google_id: Option<String>,
    /// Field representing column `created`
    pub created: i64,
}

/// Update Struct for a row in table `user_data` for [`UserData`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::AsChangeset, PartialEq, Default)]
#[diesel(table_name=user_data)]
pub struct UpdateUserData {
    /// Field representing column `username`
    pub username: Option<String>,
    /// Field representing column `nickname`
    pub nickname: Option<String>,
    /// Field representing column `password`
    pub password: Option<String>,
    /// Field representing column `email`
    pub email: Option<String>,
    /// Field representing column `discord_id`
    pub discord_id: Option<Option<String>>,
    /// Field representing column `google_id`
    pub google_id: Option<Option<String>>,
    /// Field representing column `created`
    pub created: Option<i64>,
}

/// Result of a `.paginate` function
#[derive(Debug, serde::Serialize)]
pub struct PaginationResult<T> {
    /// Resulting items that are from the current page
    pub items: Vec<T>,
    /// The count of total items there are
    pub total_items: i64,
    /// Current page, 0-based index
    pub page: i64,
    /// Size of a page
    pub page_size: i64,
    /// Number of total possible pages, given the `page_size` and `total_items`
    pub num_pages: i64,
}

impl UserData {
    /// Insert a new row into `user_data` with a given [`CreateUserData`]
    pub fn create(db: &mut ConnectionType, item: &CreateUserData) -> diesel::QueryResult<Self> {
        use crate::schema::user_data::dsl::*;

        diesel::insert_into(user_data).values(item).get_result::<Self>(db)
    }

    /// Get a row from `user_data`, identified by the primary key
    pub fn read(db: &mut ConnectionType, param_uuid: uuid::Uuid) -> diesel::QueryResult<Self> {
        use crate::schema::user_data::dsl::*;

        user_data.filter(uuid.eq(param_uuid)).first::<Self>(db)
    }

    /// Update a row in `user_data`, identified by the primary key with [`UpdateUserData`]
    pub fn update(db: &mut ConnectionType, param_uuid: uuid::Uuid, item: &UpdateUserData) -> diesel::QueryResult<Self> {
        use crate::schema::user_data::dsl::*;

        diesel::update(user_data.filter(uuid.eq(param_uuid))).set(item).get_result(db)
    }

    /// Delete a row in `user_data`, identified by the primary key
    pub fn delete(db: &mut ConnectionType, param_uuid: uuid::Uuid) -> diesel::QueryResult<usize> {
        use crate::schema::user_data::dsl::*;

        diesel::delete(user_data.filter(uuid.eq(param_uuid))).execute(db)
    }
}
