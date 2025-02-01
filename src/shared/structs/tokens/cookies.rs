use std::str::FromStr;

use cookie::{time::Duration, Cookie, SameSite};
use reqwest::StatusCode;
use uuid::Uuid;
use crate::shared::{settings::REFRESH_TOKEN_LIFETIME, structs::cookies::ExtraCookie};

use axum_extra::extract::CookieJar;


const COOKIE_REFRESH_TOKEN : &'static str = "REFRESH_TOKEN";


pub trait TokenCookie {
    fn put_rtid(self, rtid: Uuid) -> Self;
    fn get_rtid(&self) -> Result<Uuid, StatusCode>;
    fn rm_rtid(self) -> Self;
}

impl TokenCookie for CookieJar {
    fn put_rtid(self, rtid: Uuid) -> Self {
        let mut refresh = Cookie::new(COOKIE_REFRESH_TOKEN, rtid.to_string());
        refresh.set_secure(true);
        refresh.set_same_site(SameSite::Strict);
        refresh.set_http_only(true);
        refresh.set_max_age(Duration::seconds(REFRESH_TOKEN_LIFETIME as i64));
        refresh.set_path("/api/auth");
        //todo: refresh.set_domain("kaiv.space");
        self.add(refresh)
    }
    fn get_rtid(&self) -> Result<Uuid, StatusCode> {
        let cookie = self.get(COOKIE_REFRESH_TOKEN).ok_or(StatusCode::UNAUTHORIZED)?;
        Uuid::parse_str(cookie.value()).map_err(|_| StatusCode::UNAUTHORIZED)
    }
    fn rm_rtid(self) -> Self {
        //todo: clears value, need testing. maybe set_max_age(0)?
        self.remove(Cookie::from(COOKIE_REFRESH_TOKEN))
    }
}






