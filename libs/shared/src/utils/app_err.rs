use axum::{body::Body, response::{IntoResponse, Response}};
use reqwest::StatusCode;
use tracing::error;

pub struct AppErr(pub anyhow::Error);

impl AppErr {
    pub fn default() -> Self {
        AppErr(anyhow::Error::msg("Error"))
    }
}

impl IntoResponse for AppErr {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
        ).into_response()
    }
}

impl<E> From<E> for AppErr
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub trait ToResponseBody<T> {
    fn trough_app_err(self) -> Result<T, Response<Body>>;
}

impl<T> ToResponseBody<T> for anyhow::Result<T, anyhow::Error> 
{
    fn trough_app_err(self) -> Result<T, Response<Body>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(AppErr(e).into_response())
        }
    }
}

impl<T> ToResponseBody<T> for Result<T, String> {
    fn trough_app_err(self) -> Result<T, Response<Body>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) =>{
                error!("{}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                ).into_response()
            )},
        }
    }
}