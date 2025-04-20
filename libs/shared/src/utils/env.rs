#![allow(non_snake_case)]
#![allow(unused)]

use std::str::FromStr;

#[derive(Debug)]
pub enum ParseError {
    Missing,
    Invalid,
}

impl ParseError {
    pub fn describe_panic(&self, name: &'static str, ty: &'static str) -> ! {
        match self {
            Self::Invalid => panic!("Invalid env var: {} - must be {}", name, ty),
            Self::Missing => panic!("Missing required env var: {}", name)
        }
    }
}


pub trait TryParse<E> {
    fn try_parse<T : std::str::FromStr>(self) -> Result<T, E>;
}

impl<E> TryParse<ParseError> for Result<String, E> {
    fn try_parse<T: std::str::FromStr>(self) -> Result<T, ParseError> {
        match self {
            Ok(v) => v.parse::<T>().ok().ok_or(ParseError::Invalid),
            Err(_) => Err(ParseError::Missing),
        }
    }
}

pub trait Operator<T, E> {
    fn if_none(self, rh: Result<T, E>) -> Result<T, E>;
}

impl<T ,E> Operator<T, E> for () {
    fn if_none(self, rh: Result<T, E>) -> Result<T, E> {
        rh 
    }
}

impl<T> Operator<T, ParseError> for (T,) {
    fn if_none(self, rh: Result<T, ParseError>) -> Result<T, ParseError> {
        match rh {
            Ok(v) => Ok(v),
            Err(_e) => Ok(self.0),
        }
    }
}



#[macro_export]
macro_rules! env_config {
    ($($filename:expr => $glob:ident = $struct:ident {$($field:ident : $type:ty $(= $op_val:expr)? ),* $(,)?})*) => {
        $(
            #[allow(non_snake_case)]
            pub(crate) struct $struct {
                $(pub $field: $type),*
            }
            impl $struct {
                fn new() -> Self {
                    Self {
                        $(
                            $field: 
                            $crate::utils::env::Operator::if_none(($($op_val,)?), 
                            $crate::utils::env::TryParse::try_parse::<$type>(std::env::var(stringify!($field).to_ascii_uppercase()))
                            ).unwrap_or_else(|e| e.describe_panic(stringify!($field), stringify!($type))),
                        )*
                    }
                }
            }

            pub(crate) static $glob : $crate::once_cell::sync::Lazy<$struct> = $crate::once_cell::sync::Lazy::new(|| {
                $crate::dotenvy::from_filename_override($filename).ok(); // only for develop
                $struct::new()
            });
        )*
    };
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env::{set_var, remove_var};
    
    #[test]
    fn test_cfg() {
        set_var("TEST1_STRING", "test");
        set_var("TEST1_INT", "123");
        set_var("TEST1_BOOL", "false");

        env_config!{
            "" => CFG = Config {
                TEST1_STRING: String,
                TEST1_INT: i16,
                TEST1_BOOL: bool,
            }
        }

        assert_eq!(CFG.TEST1_STRING, std::env::var("TEST1_STRING").unwrap());
        assert_eq!(CFG.TEST1_INT, std::env::var("TEST1_INT").unwrap().parse::<i16>().unwrap());
        assert_eq!(CFG.TEST1_BOOL, std::env::var("TEST1_BOOL").unwrap().parse::<bool>().unwrap());
    }

    #[test]
    fn test_default_value() {
        remove_var("VAR_WITH_DEFAULT");
        env_config!{
            "" => CFG_DEFAULT = ConfigDefault {
                VAR_WITH_DEFAULT: i32 = 100,
            }
        }
        assert_eq!(CFG_DEFAULT.VAR_WITH_DEFAULT, 100);
    }

    #[test]
    fn test_env_over_default_value() {
        set_var("VAR_OVER_DEFAULT", "200");
        env_config!{
            "" => CFG_DEFAULT_ENV = ConfigDefaultEnv {
                VAR_OVER_DEFAULT: i32 = 100,
            }
        }
        assert_eq!(CFG_DEFAULT_ENV.VAR_OVER_DEFAULT, 200);
        remove_var("VAR_OVER_DEFAULT");
    }

    #[test]
    #[should_panic(expected = "Missing required env var: VAR_MISSING")]
    fn test_missing_env() {
        remove_var("VAR_MISSING");
        env_config!{
            "" => CFG_MISSING = ConfigMissing {
                VAR_MISSING: i32,
            }
        }
        let _ = CFG_MISSING.VAR_MISSING;
    }

    #[test]
    #[should_panic(expected = "Invalid env var: VAR_INVALID - must be i32")]
    fn test_invalid_value() {
        set_var("VAR_INVALID", "not_a_number");
        env_config!{
            "" => CFG_INVALID = ConfigInvalid {
                VAR_INVALID: i32,
            }
        }
        let _ = CFG_INVALID.VAR_INVALID;
        remove_var("VAR_INVALID");
    }
}