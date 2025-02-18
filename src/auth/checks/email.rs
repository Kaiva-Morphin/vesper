use reqwest::StatusCode;
use crate::models::user_data::UserData;
use crate::schema::user_data::dsl::*;
use crate::shared::structs::app_state::postgre::Postgre;
use diesel::prelude::*;




pub async fn is_email_available(postgre: &Postgre, email_string: String) -> Result<bool, StatusCode> {
    Ok(postgre.interact(move | conn| {
        user_data.select(UserData::as_select())
            .filter(email.eq(&email_string))
            .load(conn).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
    }).await?.len() == 0)
}
