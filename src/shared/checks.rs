use regex::Regex;

pub trait UsernameValidation {
    fn is_username_valid(&self) -> bool;
}

impl UsernameValidation for String {
    fn is_username_valid(&self) -> bool {
        let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        re.is_match(self)
    }
}
