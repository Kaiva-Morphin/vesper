use std::env;

use axum::{extract::Query, response::{IntoResponse, Redirect}};
use oauth2::{basic::BasicClient, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl};
use serde::{Deserialize, Serialize};
use crate::shared::env::{GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_REDIRECT_URI};
use super::shared::{AuthCallback, UserInfo};


pub fn google_oauth_client() -> Result<BasicClient, oauth2::url::ParseError> {
    Ok(BasicClient::new(
        ClientId::new(GOOGLE_CLIENT_ID.to_string()),
        Some(ClientSecret::new(GOOGLE_CLIENT_SECRET.to_string())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(GOOGLE_REDIRECT_URI.to_string())?))
}


pub async fn google_login() -> impl IntoResponse {
    let client = google_oauth_client().unwrap();
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();
    Redirect::temporary(auth_url.to_string().as_str())
}




pub async fn google_callback(Query(params): Query<AuthCallback>) -> impl IntoResponse {
    let client = google_oauth_client().unwrap();
    let token = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(::oauth2::reqwest::async_http_client)
        .await
        .unwrap();
    format!("Access Token: {:?}\n Info: {:?}", token, fetch_google_user_info(token.access_token().secret()).await)
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    sub: String,
    picture: String,
    email: String,
    email_verified: bool,
    given_name: String,
    name: String
}

impl Into<UserInfo> for GoogleUserInfo {
    fn into(self) -> UserInfo {
        UserInfo {
            service: super::shared::Service::Google,
            id: self.sub,
            name: self.given_name,
            nickname: self.name,
            avatar: self.picture,
            email: self.email,
            verified: self.email_verified,
        }
    }
}

pub async fn fetch_google_user_info(access_token: &str) -> Result<GoogleUserInfo, reqwest::Error> {
    let userinfo_url = "https://www.googleapis.com/oauth2/v3/userinfo";
    let client = reqwest::Client::new();
    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<GoogleUserInfo>()
        .await?;
    Ok(response)
}