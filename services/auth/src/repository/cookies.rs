use std::str::FromStr;

use cookie::{time::Duration, Cookie, SameSite};

use axum_extra::extract::CookieJar;

use crate::{CFG, ENV};


const COOKIE_REFRESH_TOKEN : &'static str = "REFRESH_TOKEN";



pub trait ExtraCookie {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T>;
}
impl ExtraCookie for CookieJar {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T> {
        self.get(name).and_then(|cookie| cookie.value().parse::<T>().ok())
    }
    // fn put_user(self, user: AnonymousUser) -> Self {
    //     self.add(Cookie::new(COOKIE_PUBLIC_ID, user.public_id.to_string()))
    //         .add(Cookie::new(COOKIE_PRIVATE_ID, user.id.to_string()))
    // }
    // fn get_passwd_hash(&self) -> Option<String> {
    //     self.get(COOKIE_PASSWORD_HASH).map(|v| v.value().to_string())
    // }
    // fn get_user_id(&self) -> Option<UserId> {
    //     self.get_typed(COOKIE_PRIVATE_ID)
    // }
    // fn get_public_user_id(&self) -> Option<PublicUserId> {
    //     self.get_typed(COOKIE_PUBLIC_ID)
    // }
    // fn get_room_id(&self) -> Option<RoomId> {
    //     self.get_typed(COOKIE_ROOM_ID)
    // }
}

pub trait TokenCookie {
    fn put_refresh(self, token: String) -> Self;
    fn get_refresh(&self) -> Option<String>;
    fn rm_refresh(self) -> Self;
}

impl TokenCookie for CookieJar {
    fn put_refresh(self, token: String) -> Self {
        let mut refresh = Cookie::new(COOKIE_REFRESH_TOKEN, token);
        refresh.set_secure(true);
        refresh.set_same_site(SameSite::Strict);
        refresh.set_http_only(true);
        refresh.set_max_age(Duration::seconds(CFG.REFRESH_TOKEN_LIFETIME as i64));
        //todo: refresh.set_path("/api/auth");
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



