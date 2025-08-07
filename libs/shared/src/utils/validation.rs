use once_cell::sync::Lazy;
use regex::Regex;

use crate::CFG;


pub const COMPILED_UID_REGEX : Lazy<Regex> = Lazy::new(||Regex::new(r"^([a-z0-9_]){3,24}$").expect("Can't compile login regex!"));
pub const COMPILED_EMAIL_REGEX : Lazy<Regex> = Lazy::new(||Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("Can't compile email regex!"));
pub const COMPILED_PASSWORD_REGEX : Lazy<Regex> = Lazy::new(||Regex::new(r"^([A-Za-z0-9_\-+=\$#~@*;:.<>/\\|!]){8,32}$").expect("Can't compile password regex!"));


pub trait RegisterValidations {
    fn is_uid_valid(&self) -> bool;
    fn is_email_valid(&self) -> bool;
    fn is_password_valid(&self) -> bool;
    fn is_nickname_valid(&self) -> bool;
}

impl RegisterValidations for String {
    fn is_uid_valid(&self) -> bool {
        COMPILED_UID_REGEX.is_match(self)
    }
    fn is_email_valid(&self) -> bool {
        COMPILED_EMAIL_REGEX.is_match(self)
    }
    fn is_password_valid(&self) -> bool {
        COMPILED_PASSWORD_REGEX.is_match(self)
    }
    fn is_nickname_valid(&self) -> bool { 
        let len = self.trim().chars().count();
        len >= CFG.MIN_NICKNAME_LENGTH && len <= CFG.MAX_NICKNAME_LENGTH
    }
}