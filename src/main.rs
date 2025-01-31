use axum::{
    extract::Query, response::{IntoResponse, Redirect}, routing::{get, post}, Json, Router
};
use google_gmail1::{api::Message, hyper, hyper_rustls, Error, Gmail};
use ::oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;
use std::{fs, sync::Arc};

use auth::{oauth::{discord_oauth_client, google_oauth_client}, refresh::refresh_tokens};
use auth::login::login;
use auth::register::register;

// SECRETS
const DB_USERNAME : &'static str = "root";
const DB_PASSWORD : &'static str = "root";
const REFRESH_TOKEN_SECRET : &[u8] = "refresh_secret".as_bytes();
const ACCESS_TOKEN_SECRET : &[u8] = "access_secret".as_bytes();
// VARIABLES
const DB_NAMESPACE : &'static str = "ns";
const DB_DATABASE : &'static str = "db";
const DB_ADDRESS : &'static str = "127.0.0.1:8000";

const REFRESH_TOKEN_LIFETIME: u64 = 60 * 60 * 24 * 15;
const ACCESS_TOKEN_LIFETIME: u64 = 60 * 10;

const MIN_USERNAME_LENGTH: usize = 2;
const MAX_USERNAME_LENGTH: usize = 24;
const MIN_PASSWORD_LENGTH: usize = 6;
const MAX_PASSWORD_LENGTH: usize = 64;

const CONTACT_ADMIN_MESSAGE : &'static str = "Notify the administrator if the error does not resolve itself after some time";




#[tokio::main]
async fn main() -> surrealdb::Result<()>{

    /*let secret= oauth2::ApplicationSecret{
        client_id: "385348935365-2qm7kpo5kkbu639e9su7jh5jo44tnk9s.apps.googleusercontent.com".to_string(),
        client_secret: "***REMOVED***".to_string(),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        redirect_uris: vec![],
        project_id: Some("balmy-parser-435823-i8".to_string()),
        client_email: None,
        auth_provider_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_string()),
        client_x509_cert_url: None
    };
        
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    ).build().await.unwrap();
    let mut hub = Gmail::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().unwrap().https_or_http().enable_http1().build()), auth);
    let mut req = Message::default();
    

    // You can configure optional parameters by calling the respective setters at will, and
    // execute the final call using `upload_resumable(...)`.
    // Values shown here are possibly random and not representative !
    let result = hub.users().messages_import(req, "111vadimchik111@gmail.com")
                .process_for_calendar(true)
                .never_mark_spam(false)
                .internal_date_source("amet.")
                .deleted(true)
                .upload_resumable(fs::File::open("C:\\Users\\kaiv\\Videos\\2024-07-26 12-02-26.mkv").unwrap(), "application/octet-stream".parse().unwrap()).await;

    match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            |Error::Io(_)
            |Error::MissingAPIKey
            |Error::MissingToken(_)
            |Error::Cancelled
            |Error::UploadSizeLimitExceeded(_, _)
            |Error::Failure(_)
            |Error::BadRequest(_)
            |Error::FieldClash(_)
            |Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res),
    }*/


    let db = Surreal::new::<Ws>(DB_ADDRESS).await.unwrap();
    db.signin(Root {
        username: DB_USERNAME,
        password: DB_PASSWORD,
    })
    .await
    .unwrap();
    db.use_ns(DB_NAMESPACE).use_db(DB_DATABASE).await.unwrap();
    let db = Arc::new(db);
    let app = Router::new()
        .route("/register", post(register))
        .route("/refresh_tokens", post(refresh_tokens))
        .route("/login", post(login))

        .route("/auth/google/login", get(google_login))
        .route("/auth/google/callback", get(google_callback))
        .route("/auth/discord/login", get(discord_login))
        .route("/auth/discord/callback", get(discord_callback))

        .with_state(db)
        ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}









async fn google_login() -> impl IntoResponse {
    let client = google_oauth_client();
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();
    Redirect::temporary(auth_url.to_string().as_str())
}

#[derive(Deserialize)]
struct AuthCallback {
    code: String,
    state: String,
}

async fn google_callback(Query(params): Query<AuthCallback>) -> impl IntoResponse {
    let client = google_oauth_client();
    let token = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(::oauth2::reqwest::async_http_client)
        .await
        .unwrap();
    format!("Access Token: {:?}\n Info: {:?}", token, fetch_google_user_info(token.access_token().secret()).await)
}

async fn discord_login() -> impl IntoResponse {
    let client = discord_oauth_client();
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();
    Redirect::temporary(auth_url.to_string().as_str())
}

async fn discord_callback(Query(params): Query<AuthCallback>) -> impl IntoResponse {
    let client = discord_oauth_client();
    let token = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(::oauth2::reqwest::async_http_client)
        .await
        .unwrap();
    format!("Access Token: {:?}\n Info: {:?}", token, fetch_discord_user_info(token.access_token().secret()).await)
}

async fn fetch_google_user_info(access_token: &str) -> Result<serde_json::Value, reqwest::Error> {
    let userinfo_url = "https://www.googleapis.com/oauth2/v3/userinfo";
    let client = reqwest::Client::new();

    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(response)
}


async fn fetch_discord_user_info(access_token: &str) -> Result<serde_json::Value, reqwest::Error> {
    let userinfo_url = "https://discord.com/api/users/@me";
    let client = reqwest::Client::new();

    let response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(response)
}



