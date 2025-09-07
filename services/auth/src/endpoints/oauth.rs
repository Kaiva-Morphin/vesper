
use std::collections::HashMap;

use axum::{extract::{Query, State}, http::StatusCode, response::{IntoResponse, Redirect}, Extension, Json};
use axum_extra::extract::CookieJar;
use bb8_redis::redis::AsyncCommands;
use layers::logging::UserInfoExt;
use oauth2::{basic::{BasicClient, BasicErrorResponseType, BasicTokenType}, AuthorizationCode, ClientId, CsrfToken, EmptyExtraTokenFields, EndpointNotSet, EndpointSet, RedirectUrl, RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse};
use postgre_entities::user_data;
use redis_utils::redis::RedisConn;
use serde::{Deserialize, Serialize};
use shared::{tokens::jwt::RefreshRules, utils::{app_err::AppErr, validation::RegisterValidations}, uuid::Uuid};

use crate::{repository::{db::OauthLogin, tokens::{generate_access, generate_and_put_refresh}}, AppState, ENV};
use anyhow::Result;

use oauth2::ClientSecret;
use oauth2::TokenUrl;
use oauth2::AuthUrl;
use tracing::{error};


#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Google,
    Discord
}

#[derive(Deserialize, Debug)]
pub struct OauthCallbackQuery {
    pub provider: Provider,
    pub code: Option<String>,
    pub state: String,
    pub _scope: Option<String>,
    pub _authuser: Option<String>,
    pub _prompt: Option<String>,
}

pub type GoogleClient = oauth2::Client<
    StandardErrorResponse<BasicErrorResponseType>, 
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, 
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>, 
    StandardRevocableToken, 
    StandardErrorResponse<RevocationErrorResponseType>, EndpointSet, EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet
>;

pub fn build_google_client() -> GoogleClient {
    BasicClient::new(ClientId::new(ENV.GOOGLE_CLIENT_ID.clone()))
        .set_client_secret(ClientSecret::new(ENV.GOOGLE_CLIENT_SECRET.clone()))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap())
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap())
        .set_redirect_uri(RedirectUrl::new(ENV.GOOGLE_REDIRECT_URI.clone()).unwrap())
}



pub async fn login_google(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>
) -> axum::response::Redirect {
    let token = params.get("state").cloned().unwrap_or_default(); 
    let csrf_token = CsrfToken::new(token);
    let (auth_url, _csrf_token) = state.google_client
        .authorize_url(|| csrf_token)
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .url();
    axum::response::Redirect::temporary(auth_url.as_ref())
}

pub async fn login_discord() -> axum::response::Redirect {
    axum::response::Redirect::temporary(&ENV.DISCORD_AUTH_URI)
}


#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    email_verified: bool,
    name: String,
    picture: String,
    given_name: Option<String>,
    family_name: Option<String>,
    locale: Option<String>,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct TempRegistrationToken {
    email: String,
    picture: Option<String>,
    google_id: Option<String>,
    discord_id: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TempLoginToken {
    email: String,
    uid: Uuid,
    rules: RefreshRules,
}


pub trait TempTokenStore {
    async fn get_temp_register(&self, id: &str) -> Result<Option<TempRegistrationToken>>;
    async fn rm_temp(&self, id: &str) -> Result<()>;
    async fn put_temp_register(&self, id: &str, token: TempRegistrationToken) -> Result<()>;

    async fn get_temp_login(&self, id: &str) -> Result<Option<TempLoginToken>>;
    async fn put_temp_login(&self, id: &str, token: TempLoginToken) -> Result<()>;
}

fn id_to_key(id: &str) -> String {
    format!("TEMP_TOKEN:{id}")
}

impl TempTokenStore for RedisConn {
    async fn rm_temp(&self, id: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _ : () = conn.del(id_to_key(id)).await?;
        Ok(())
    }

    async fn get_temp_register(&self, id: &str) -> Result<Option<TempRegistrationToken>> {
        let mut conn = self.pool.get().await?;
        let r : Option<String> = conn.get(id_to_key(id)).await?;
        let r = r.and_then(|i| serde_json::from_str::<TempRegistrationToken>(&i).ok());
        Ok(r)
    }
    
    async fn put_temp_register(&self, id: &str, token: TempRegistrationToken) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _ : () = conn.set_ex(id_to_key(id), serde_json::to_string(&token)?, 5 * 60).await?;
        Ok(())
    }

    async fn get_temp_login(&self, id: &str) -> Result<Option<TempLoginToken>> {
        let mut conn = self.pool.get().await?;
        let r : Option<String> = conn.get(id_to_key(id)).await?;
        let r = r.and_then(|i| serde_json::from_str::<TempLoginToken>(&i).ok());
        Ok(r)
    }
    
    async fn put_temp_login(&self, id: &str, token: TempLoginToken) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let _ : () = conn.set_ex(id_to_key(id), serde_json::to_string(&token)?, 5 * 60).await?;
        Ok(())
    }
}



pub async fn oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<OauthCallbackQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let action: Result<TempLoginToken, TempRegistrationToken> = match query.provider {
        Provider::Google => {
            let code = query.code.ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing code").into_response())?;

            let client = reqwest::Client::new();
            
            let Ok(token_result) = state.google_client
                .exchange_code(AuthorizationCode::new(code))
                .request_async(&client)
                .await 
            else {
                return Err((StatusCode::BAD_REQUEST, "Missing code").into_response())
            };
            
            let access_token = token_result.access_token().secret();
            let user_info : GoogleUserInfo  = client
                .get("https://www.googleapis.com/oauth2/v3/userinfo")
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| {
                    error!("User info fetch failed: {e:?}");
                    (StatusCode::BAD_GATEWAY, "Google user info error").into_response()
                })?
                .json()
                .await
                .map_err(|e| {
                    error!("User info decode failed: {e:?}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to decode user info").into_response()
                })?;
            match state
                    .login_oauth(user_data::Column::GoogleId, &user_info.sub, &user_info.email)
                    .await
                    .map_err(|_|(StatusCode::INTERNAL_SERVER_ERROR).into_response())? {
                OauthLogin::Successful(uid, rules) =>
                    Ok(TempLoginToken{
                        email: user_info.email,
                        rules,
                        uid
                    }),
                OauthLogin::NeedRegistration =>
                    Err(TempRegistrationToken {
                        email: user_info.email,
                        google_id: Some(user_info.sub),
                        picture: Some(user_info.picture),
                        discord_id: None
                    }),
                OauthLogin::EmailExists => {
                    tracing::info!("Email exists");
                    return Ok(Redirect::temporary("/oauth?err=Email%20already%20used.%20Please%20log%20in%20and%20link%20your%20account.").into_response());
                }
            }
        },
        Provider::Discord => {
            // let code = query.code.ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing code").into_response())?;
            // let user_info = reqwest::Client::new()
            //     .post("POST https://discord.com/api/oauth2/token")
            //     .bearer_auth(access_token.secret())
            //     .send()
            //     .await?
            //     // .json::<GoogleUserInfo>()
            //     // .await?
            ;

            // info!("{user_info:#?}")
            todo!()

        }
    };

    let id = shared::utils::token::generate_secure_token(256);
    match action {
        Ok(l) => {
            tracing::info!("Login");
            state.redis.put_temp_login(&id.to_string(), l).await.map_err(|_|(StatusCode::INTERNAL_SERVER_ERROR).into_response())?;
            Ok(Redirect::temporary(&format!("/oauth?login={id}&state={}", query.state)).into_response())
        }
        Err(r) => {
            tracing::info!("Register");
            state.redis.put_temp_register(&id.to_string(), r).await.map_err(|_|(StatusCode::INTERNAL_SERVER_ERROR).into_response())?;
            Ok(Redirect::temporary(&format!("/oauth?register={id}&state={}", query.state)).into_response())
        }
    }
}

#[derive(Deserialize)]
pub struct FinishRegistrationRequest {
    tos_accepted: bool,
    uid: String,
    password: String,
    temp_token: String,
    turnstile_token: String
}


impl FinishRegistrationRequest {
    fn validate(&self) -> Result<(), &'static str> {
        if !self.uid.is_uid_valid() {return Err("Invalid uid!")}
        if !self.password.is_password_valid() {return Err("Invalid password!")}
        Ok(())
    }
}




pub async fn oauth_register(
    State(state): State<AppState>,
    jar: CookieJar,
    Extension(user_info) : Extension<UserInfoExt>,
    Json(req): Json<FinishRegistrationRequest>,
) -> Result<impl IntoResponse, AppErr>  {
    if !req.tos_accepted {return Ok((StatusCode::BAD_REQUEST, "Accept ToS!").into_response())};
    if let Err(msg) = req.validate() {return Ok((StatusCode::BAD_REQUEST, msg).into_response())};
    #[cfg(not(feature = "disable_turnstile"))]
    if !verify_turnstile(request_body.turnstile_token.clone(), get_user_ip(&headers)).await {return Ok((StatusCode::BAD_REQUEST, "Turnstile failed").into_response())};
    let Some(stored) = state.redis.get_temp_register(&req.temp_token).await? else {return Ok((StatusCode::UNAUTHORIZED).into_response())};
    let r = state.register_user(req.uid.clone(), req.uid, stored.email.clone(), req.password, stored.google_id, stored.discord_id).await?;
    let Ok((user_id, rules)) = r else {
        return Ok((StatusCode::CONFLICT, r.err().unwrap()).into_response())
    };
    let jar = generate_and_put_refresh(jar, &state, &user_id, user_info, stored.email, rules).await?;
    let access_response = generate_access(user_id)?;
    state.redis.rm_temp(&req.temp_token).await.ok();
    Ok((jar, access_response).into_response())
}


#[derive(Deserialize)]
pub struct TokenRequest {
    token: String,
}

pub async fn oauth_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Extension(user_info) : Extension<UserInfoExt>,
    Json(TokenRequest { token }): Json<TokenRequest>
) -> Result<impl IntoResponse, AppErr>  {
    let Some(stored) = state.redis.get_temp_login(&token).await? else {return Ok((StatusCode::UNAUTHORIZED).into_response())};
    let jar = generate_and_put_refresh(jar, &state, &stored.uid, user_info, stored.email, stored.rules).await?;
    let access_response = generate_access(stored.uid)?;
    state.redis.rm_temp(&token).await.ok();
    Ok((jar, access_response).into_response())
}
