use std::sync::Arc;

use axum::{body::Body, extract::State, http::{Response, StatusCode}, response::IntoResponse, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{CONTACT_ADMIN_MESSAGE, REFRESH_TOKEN_LIFETIME, REFRESH_TOKEN_SECRET};

use super::shared::{RefreshTokenRecord, TokenPayload, Tokens};






#[derive(Default, Debug, Serialize, Deserialize)]
struct UpdateTokenRecord {
    refresh_token: String,
    expires: u64
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RefreshTokensPayload {
    refresh_token: String
}

async fn get_token_record(db: &Arc<Surreal<Client>>, token: String) -> Result<RefreshTokenRecord, Response<Body>>{
    let res = db.query(format!("SELECT * FROM refresh_tokens WHERE refresh_token = '{token}'")).await;
    let Ok(mut res) = res else {return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant get data from db! {}", CONTACT_ADMIN_MESSAGE)).into_response())};
    let Ok(Some(record)) = res.take::<Option<RefreshTokenRecord>>(0) 
    else {return Err((StatusCode::UNAUTHORIZED, "Incorrect refresh token!").into_response());};
    println!("{:?}", record);
    if record.expires < Utc::now().timestamp() as u64 {
        let _ : Result<Option<RefreshTokenRecord>, surrealdb::Error> = db.delete(("refresh_tokens", record.uuid)).await;
        return Err((StatusCode::UNAUTHORIZED, "Incorrect refresh token!").into_response())
    }
    Ok(record)
}

pub async fn refresh_tokens(
    State(db): State<Arc<Surreal<Client>>>,
    payload: Option<Json<RefreshTokensPayload>>
) -> impl IntoResponse {
    let Some(payload) = payload else { return (StatusCode::BAD_REQUEST, "Incorrect form sent!").into_response()};
    let validation = jsonwebtoken::Validation::default();
    let token_data = jsonwebtoken::decode::<TokenPayload>(&payload.refresh_token, &jsonwebtoken::DecodingKey::from_secret(REFRESH_TOKEN_SECRET), &validation);
    let Ok(token_data) = token_data
    else {return (StatusCode::UNAUTHORIZED, "Incorrect refresh token!").into_response()};

    let record = match get_token_record(&db, payload.refresh_token.clone()).await {Ok(v) => v, Err(r) => return r};

    let now = Utc::now().timestamp() as u64;
    if token_data.claims.exp < now {return (StatusCode::UNAUTHORIZED, "Refresh token expired!").into_response()};
    
    let username = token_data.claims.user.clone();

    let Ok(tokens) = Tokens::get_pair(username.clone())
    else {return (StatusCode::INTERNAL_SERVER_ERROR, format!("Server cant generate tokens! {}", CONTACT_ADMIN_MESSAGE)).into_response()};

    let _ = db.update::<Option<RefreshTokenRecord>>(("refresh_tokens", record.uuid)).merge(
        UpdateTokenRecord{
            refresh_token: tokens.refresh.clone(),
            expires: now + REFRESH_TOKEN_LIFETIME
        }
    ).await;

    (StatusCode::OK,
    Json(tokens)).into_response()
}




