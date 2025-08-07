use axum::response::IntoResponse;
use shared::{tokens::jwt::TokenEncoder, utils::app_err::{AppErr, ToResponseBody}};


pub async fn get_timestamp() -> Result<impl IntoResponse, AppErr> {
    Ok(TokenEncoder::encode_timestamp(chrono::Utc::now().timestamp()).trough_app_err().into_response())
}