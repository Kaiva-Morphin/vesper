use deadpool_diesel::InteractError;
use reqwest::StatusCode;




pub trait AsStatusCode {
    fn as_interaction_error(&self) -> StatusCode;
}

impl AsStatusCode for diesel::result::Error {
    fn as_interaction_error(&self) -> StatusCode {
        match self {
            diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl AsStatusCode for deadpool_diesel::PoolError {
    fn as_interaction_error(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl AsStatusCode for InteractError {
    fn as_interaction_error(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub fn adapt_error<T: AsStatusCode>(err: T) -> StatusCode {
    err.as_interaction_error()
}