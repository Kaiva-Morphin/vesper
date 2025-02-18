use reqwest::StatusCode;
use crate::models::user_data::UserData;
use crate::schema::user_data::dsl::*;
use crate::shared::structs::app_state::postgre::Postgre;
use diesel::prelude::*;




pub async fn is_username_available(postgre: &Postgre, name: String) -> Result<bool, StatusCode> {
    Ok(postgre.interact(move | conn| {
        user_data.select(UserData::as_select())
            .filter(username.eq(&name))
            .load(conn).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
    }).await?.len() == 0)
}
