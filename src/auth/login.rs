use std::sync::Arc;

use axum::{
    extract::State, http::StatusCode, response::IntoResponse, Json
};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::CONTACT_ADMIN_MESSAGE;

use super::shared::{create_token_record, Tokens, UserData, UsernameValidation};



#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    username: String,
    password: String
}



pub async fn login(
    State(db): State<Arc<Surreal<Client>>>,
    payload: Option<Json<LoginBody>>
) -> impl IntoResponse {
    let Some(payload) = payload else { return (StatusCode::BAD_REQUEST, "Incorrect form sent!").into_response()};
    if !payload.username.is_username_valid() {return (StatusCode::BAD_REQUEST, "Incorrect username!").into_response()};
    let Ok(user_data) = db.select::<Option<UserData>>(("user_data", &payload.username)).await
    else {return (StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant get data from db! {}", CONTACT_ADMIN_MESSAGE)).into_response()};

    let Some(user_data) = user_data
    else {return (StatusCode::UNAUTHORIZED, "Incorrect username password pair!").into_response()};

    let Ok(res) = bcrypt::verify(payload.password.clone(), user_data.password.as_str())
    else {return (StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant hash password! {}", CONTACT_ADMIN_MESSAGE)).into_response()};

    if !res {return (StatusCode::UNAUTHORIZED, "Incorrect username password pair!").into_response()};

    let Ok(tokens) = Tokens::get_pair(payload.username.clone())
    else {return (StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant generate tokens! {}", CONTACT_ADMIN_MESSAGE)).into_response()};
    if let Err(r) = create_token_record(&db, payload.username.clone(), tokens.refresh.clone()).await {
        return r;
    }
    (StatusCode::OK,
    Json(tokens)).into_response()
}


