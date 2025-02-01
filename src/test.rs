use axum::{routing::post, Json, Router};
use axum_extra::extract::CookieJar;
use cookie::{time::Duration, Cookie};
use redis::{Commands, RedisResult, ErrorKind};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/test", post(refresh_tokens));

    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:1234")).await.unwrap();
    println!("{}", listener.local_addr().expect("failed to return local address"));
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct Body{create: bool}
#[derive(Serialize, Deserialize)]

pub struct Resp {
    msg: String
}

pub async fn refresh_tokens(
    jar: CookieJar,
    payload: Json<Body>,
) -> Result<(CookieJar, Json<Resp>), (CookieJar, StatusCode)> {
    if payload.create{
        Ok(
            (jar.add(Cookie::new("A", "B")), Json(Resp{msg: "added".to_string()}))
        )
    } else {
        let mut c = Cookie::from("A");
        c.set_max_age(Duration::seconds(0));
        Err((jar.add(c), StatusCode::INTERNAL_SERVER_ERROR))
    }
}