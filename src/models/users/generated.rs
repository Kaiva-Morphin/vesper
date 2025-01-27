/* @generated and managed by dsync */

#[allow(unused)]
use crate::diesel::*;
use crate::schema::*;

pub type ConnectionType = diesel::pg::PgConnection;

/// Struct representing a row in table `users`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Queryable, diesel::Selectable, diesel::QueryableByName, diesel::Identifiable)]
#[diesel(table_name=users, primary_key(uuid))]
pub struct Users {
    /// Field representing column `uuid`
    pub uuid: uuid::Uuid,
    /// Field representing column `refresh_token`
    pub refresh_token: String,
    /// Field representing column `expires`
    pub expires: i64,
    /// Field representing column `username`
    pub username: String,
}

/// Create Struct for a row in table `users` for [`Users`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Insertable)]
#[diesel(table_name=users)]
pub struct CreateUsers {
    /// Field representing column `uuid`
    pub uuid: uuid::Uuid,
    /// Field representing column `refresh_token`
    pub refresh_token: String,
    /// Field representing column `expires`
    pub expires: i64,
    /// Field representing column `username`
    pub username: String,
}

/// Update Struct for a row in table `users` for [`Users`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::AsChangeset, PartialEq, Default)]
#[diesel(table_name=users)]
pub struct UpdateUsers {
    /// Field representing column `refresh_token`
    pub refresh_token: Option<String>,
    /// Field representing column `expires`
    pub expires: Option<i64>,
    /// Field representing column `username`
    pub username: Option<String>,
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

impl Users {
    /// Insert a new row into `users` with a given [`CreateUsers`]
    pub fn create(db: &mut ConnectionType, item: &CreateUsers) -> diesel::QueryResult<Self> {
        use crate::schema::users::dsl::*;

        diesel::insert_into(users).values(item).get_result::<Self>(db)
    }

    /// Get a row from `users`, identified by the primary key
    pub fn read(db: &mut ConnectionType, param_uuid: uuid::Uuid) -> diesel::QueryResult<Self> {
        use crate::schema::users::dsl::*;

        users.filter(uuid.eq(param_uuid)).first::<Self>(db)
    }

    /// Update a row in `users`, identified by the primary key with [`UpdateUsers`]
    pub fn update(db: &mut ConnectionType, param_uuid: uuid::Uuid, item: &UpdateUsers) -> diesel::QueryResult<Self> {
        use crate::schema::users::dsl::*;

        diesel::update(users.filter(uuid.eq(param_uuid))).set(item).get_result(db)
    }

    /// Delete a row in `users`, identified by the primary key
    pub fn delete(db: &mut ConnectionType, param_uuid: uuid::Uuid) -> diesel::QueryResult<usize> {
        use crate::schema::users::dsl::*;

        diesel::delete(users.filter(uuid.eq(param_uuid))).execute(db)
    }
}
