use reqwest::Client;
use serde::Deserialize;

use crate::ENV;

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
}

pub async fn verify_turnstile(response: String, ip: String) -> bool { //todo!: checks; MOVE TO MIDDLEWARE!
    let client = Client::new();
    let response = client.post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[("secret", ENV.TURNSTILE_SECRET.clone()), ("response", response), ("remoteip", ip)])
        .send()
        .await
        .unwrap()
        .json::<TurnstileResponse>()
        .await
        .unwrap();
    response.success
}
