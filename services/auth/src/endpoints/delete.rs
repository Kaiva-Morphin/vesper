use axum::{extract::State, http::{StatusCode}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use shared::utils::app_err::AppErr;

use crate::AppState;

// TODO!
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteBody {
    pub email_or_uid: String,
    pub password: String,
    // pub email: String,
    // pub email_code: String,
    // pub turnstile_token: String,
}

pub async fn delete_account(
    State(state): State<AppState>,
    // Extension(user_info) : Extension<UserInfoExt>,
    Json(request_body): Json<DeleteBody>,
) -> Result<impl IntoResponse, AppErr> {
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_token.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::BAD_REQUEST, "Turnstile failed").into_response())};
    // #[cfg(not(feature = "disable_email"))]
    // if !state.verify_register_code(request_body.email_code.clone(), request_body.email.clone()).await? {return Ok((StatusCode::BAD_REQUEST, "Invalid email code!").into_response())};
    let success = state.delete_user(request_body.email_or_uid, request_body.password).await?;
    if !success {return Ok(StatusCode::UNAUTHORIZED.into_response())}
    Ok(().into_response())
}
