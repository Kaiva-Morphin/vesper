use oauth2::{
    AuthUrl, Client, ClientId, ClientSecret, RedirectUrl, TokenUrl
};
use std::env;

pub fn google_oauth_client() -> Result<Client, oauth2::url::ParseError> {
    Client::new(ClientId::new(env::var("GOOGLE_CLIENT_ID").unwrap()))
        .set_client_secret(ClientSecret::new(env::var("GOOGLE_CLIENT_SECRET").unwrap()))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())?)
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?)
        .set_redirect_uri(RedirectUrl::new(env::var("GOOGLE_REDIRECT_URI").unwrap())?)
}

pub fn discord_oauth_client() -> BasicClient {
    BasicClient::new(
        ClientId::new(env::var("DISCORD_CLIENT_ID").unwrap()),
        Some(ClientSecret::new(env::var("DISCORD_CLIENT_SECRET").unwrap())),
        AuthUrl::new("https://discord.com/oauth2/authorize?client_id=1313838865580032081&response_type=code&redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fdiscord%2Fcallback&scope=identify".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(env::var("DISCORD_REDIRECT_URI").unwrap()).unwrap())
}