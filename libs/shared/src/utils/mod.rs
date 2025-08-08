use uuid::Uuid;

pub mod token;
pub mod validation;
pub mod hash;
pub mod cookies;
pub mod verify_turnstile;
pub mod header;
pub mod app_err;
pub mod logger;
pub mod env;
pub mod set_encoder;




pub trait IntoKey {
    fn into_key(&self) -> String;
}

impl IntoKey for Uuid {
    fn into_key(&self) -> String {
        self.simple().to_string()
    }
}
