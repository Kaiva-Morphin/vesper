use cookie::{time::Duration, Cookie, SameSite};
use axum_extra::extract::CookieJar;
use crate::CFG;


const COOKIE_REFRESH_TOKEN : &str = "REFRESH_TOKEN";

pub trait TokenCookie {
    fn put_refresh(self, token: String) -> Self;
    fn get_refresh(&self) -> Option<String>;
    fn rm_refresh(self) -> Self;
}

impl TokenCookie for CookieJar {
    fn put_refresh(self, token: String) -> Self {
        let mut refresh = Cookie::new(COOKIE_REFRESH_TOKEN, token);
        // TODO! : DEV ONLY
        // refresh.set_secure(true);
        // refresh.set_same_site(SameSite::Strict);
        // refresh.set_http_only(true);
        refresh.set_max_age(Duration::seconds(CFG.REFRESH_TOKEN_LIFETIME as i64));
        // refresh.set_path("/api/auth");
        //todo: refresh.set_domain("kaiv.space");
        self.add(refresh)
    }

    fn get_refresh(&self) -> Option<String> {
        let cookie = self.get(COOKIE_REFRESH_TOKEN)?;
        Some(cookie.value().to_string())
    }
    
    fn rm_refresh(self) -> Self {
        //todo: clears value, need testing. maybe set_max_age(0)?
        self.remove(Cookie::from(COOKIE_REFRESH_TOKEN))
    }
}



