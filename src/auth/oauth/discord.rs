use axum::{extract::{Query, State}, response::{IntoResponse, Redirect}};
use oauth2::{basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use crate::{shared::{env::{CALLBACK_REDIRECT_URI, DISCORD_CLIENT_ID, DISCORD_CLIENT_SECRET, DISCORD_REDIRECT_URI}, replace_err::ReplaceErr}, AppState};
use super::shared::{AuthCallback, Service, UserInfo, DISCORD_PROVIDER, PROVIDER_KEY};

pub fn discord_oauth_client() -> Result<BasicClient, oauth2::url::ParseError> {
    Ok(BasicClient::new(
        ClientId::new(DISCORD_CLIENT_ID.to_string()),
        Some(ClientSecret::new(DISCORD_CLIENT_SECRET.to_string())),
        AuthUrl::new("https://discord.com/api/oauth2/authorize?response_type=code".to_string())?,
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(CALLBACK_REDIRECT_URI.to_string())?))
}

pub async fn discord_login(State(state): State<AppState>) -> Result<Redirect, StatusCode> {
    let (auth_url, csrf_token) = state.discord_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .url();
    let _ : () = state.tokens.set_crfs(csrf_token.secret(), DISCORD_PROVIDER.to_string())?;
    Ok(Redirect::temporary(auth_url.to_string().as_str()))
}



pub async fn discord_callback(client: BasicClient, code: String) -> Result<DiscordUserInfo, StatusCode> {
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await.replace_err(StatusCode::NOT_ACCEPTABLE)?;
    Ok(fetch_discord_user_info(token.access_token().secret()).await.replace_err(StatusCode::INTERNAL_SERVER_ERROR)?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUserInfo {
    id: String,
    avatar: String,
    email: String,
    username: String,
    verified: bool,
    // locale: String,
    global_name: String
}

impl Into<UserInfo> for DiscordUserInfo {
    fn into(self) -> UserInfo {
        UserInfo {
            service: Service::Discord,
            id: self.id,
            name: self.username,
            nickname: self.global_name,
            avatar: self.avatar,
            email: self.email,
            verified: self.verified,
        }
    }
}

pub async fn fetch_discord_user_info(
    access_token: &str
) -> Result<DiscordUserInfo, reqwest::Error> {
    let userinfo_url = "https://discord.com/api/users/@me";
    let client = reqwest::Client::new();
    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<DiscordUserInfo>()
        .await?;
    Ok(response)
}
