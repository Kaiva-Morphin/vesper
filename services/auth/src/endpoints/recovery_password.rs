use std::collections::HashMap;

use axum::{extract::{Query, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use shared::utils::app_err::AppErr;
use tracing::{info, warn};

use crate::{repository::email::CodeKind, AppState, CFG};

use super::register::RegisterValidations;


#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryRequest {
    pub email_or_login: String,
    pub turnstile_response: String
}

#[axum::debug_handler]
pub async fn request_password_recovery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request_body): Json<RecoveryRequest>,
) -> Result<impl IntoResponse, AppErr> {
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_response.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::BAD_REQUEST, "Turnstile failed").into_response())};
    Ok(state.try_send_recovery_code(&request_body.email_or_login).await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRequest {
    pub new_password: String,
    pub turnstile_response: String
}


#[axum::debug_handler]
pub async fn recovery_password(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(request_body): Json<UpdateRequest>,
) -> Result<impl IntoResponse, AppErr> {
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_response.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::BAD_REQUEST, "Turnstile failed").into_response())};
    if !request_body.new_password.is_password_valid() {return Ok((StatusCode::BAD_REQUEST, "Bad password!").into_response())}
    if let Some(token) = params.get("token") {
        if token.chars().count() == CFG.RECOVERY_TOKEN_LEN {
            let r = state.recovery_password(token, request_body.new_password).await?;
            if r.is_some(){
                // todo: send email
                return Ok("New password set!".into_response())
            }
        }
    }
    Ok((StatusCode::UNAUTHORIZED, "Incorrect token").into_response())

    // let fingerprint = request_body.fingerprint.clone();
    // let v = state.register_user(request_body).await?;
    // let user_id = match v {Ok(user) => user, Err(msg) => return Ok((StatusCode::CONFLICT, msg).into_response())};
    // let jar = generate_and_put_refresh(jar, &state, &user_id, fingerprint, get_user_agent(&headers), get_user_ip(&headers))?;
    // let access_response = generate_access(user_id)?;
    
}
