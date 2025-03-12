use std::str::FromStr;

use axum_extra::extract::CookieJar;


pub trait ExtraCookie {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T>;
}

impl ExtraCookie for CookieJar {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T> {
        self.get(name).and_then(|cookie| cookie.value().parse::<T>().ok())
    }
}