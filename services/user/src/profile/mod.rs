use anyhow::Result;
use axum::response::IntoResponse;

pub async fn set_profile_status() -> Result<String> {Ok("".to_string())}
pub async fn set_profile_background() -> Result<String> {Ok("".to_string())}
pub async fn set_profile_style() -> Result<String> {Ok("".to_string())}
pub async fn set_profile_visibility() -> Result<String> {Ok("".to_string())}

pub async fn set_mini_profile_status() -> Result<String> {Ok("".to_string())}
pub async fn set_mini_profile_background() -> Result<String> {Ok("".to_string())}
pub async fn set_mini_profile_style() -> Result<String> {Ok("".to_string())}

pub async fn get_profile() -> impl IntoResponse {().into_response()}
pub async fn get_mini_profile() -> impl IntoResponse{().into_response()}





