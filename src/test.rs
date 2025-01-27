use diesel;
use diesel::prelude::*;
use dotenvy::dotenv;
use models::users::Users;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[tokio::main]
async fn main() -> Result<(), ()>{
    use self::schema::users::dsl::*;
    let connection = &mut establish_connection();
    let results = users
        .filter(expires.ne(123))
        .limit(5)
        .select(Users::as_select())
        .load(connection)
        .expect("Error loading posts");
    println!("{:#?}", results);
    Ok(())
}




