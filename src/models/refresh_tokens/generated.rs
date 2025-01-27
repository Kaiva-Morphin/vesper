/* @generated and managed by dsync */

#[allow(unused)]
use crate::diesel::*;
use crate::schema::*;

pub type ConnectionType = diesel::pg::PgConnection;

/// Struct representing a row in table `refresh_tokens`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Queryable, diesel::Selectable, diesel::QueryableByName, diesel::Identifiable)]
#[diesel(table_name=refresh_tokens, primary_key(uuid))]
pub struct RefreshTokens {
    /// Field representing column `uuid`
    pub uuid: uuid::Uuid,
    /// Field representing column `refresh_token`
    pub refresh_token: String,
    /// Field representing column `expires`
    pub expires: i64,
    /// Field representing column `username`
    pub username: String,
}

/// Create Struct for a row in table `refresh_tokens` for [`RefreshTokens`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Insertable)]
#[diesel(table_name=refresh_tokens)]
pub struct CreateRefreshTokens {
    /// Field representing column `uuid`
    pub uuid: uuid::Uuid,
    /// Field representing column `refresh_token`
    pub refresh_token: String,
    /// Field representing column `expires`
    pub expires: i64,
    /// Field representing column `username`
    pub username: String,
}

/// Update Struct for a row in table `refresh_tokens` for [`RefreshTokens`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::AsChangeset, PartialEq, Default)]
#[diesel(table_name=refresh_tokens)]
pub struct UpdateRefreshTokens {
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

impl RefreshTokens {
    /// Insert a new row into `refresh_tokens` with a given [`CreateRefreshTokens`]
    pub fn create(db: &mut ConnectionType, item: &CreateRefreshTokens) -> diesel::QueryResult<Self> {
        use crate::schema::refresh_tokens::dsl::*;

        diesel::insert_into(refresh_tokens).values(item).get_result::<Self>(db)
    }

    /// Get a row from `refresh_tokens`, identified by the primary key
    pub fn read(db: &mut ConnectionType, param_uuid: uuid::Uuid) -> diesel::QueryResult<Self> {
        use crate::schema::refresh_tokens::dsl::*;

        refresh_tokens.filter(uuid.eq(param_uuid)).first::<Self>(db)
    }

    /// Update a row in `refresh_tokens`, identified by the primary key with [`UpdateRefreshTokens`]
    pub fn update(db: &mut ConnectionType, param_uuid: uuid::Uuid, item: &UpdateRefreshTokens) -> diesel::QueryResult<Self> {
        use crate::schema::refresh_tokens::dsl::*;

        diesel::update(refresh_tokens.filter(uuid.eq(param_uuid))).set(item).get_result(db)
    }

    /// Delete a row in `refresh_tokens`, identified by the primary key
    pub fn delete(db: &mut ConnectionType, param_uuid: uuid::Uuid) -> diesel::QueryResult<usize> {
        use crate::schema::refresh_tokens::dsl::*;

        diesel::delete(refresh_tokens.filter(uuid.eq(param_uuid))).execute(db)
    }
}
