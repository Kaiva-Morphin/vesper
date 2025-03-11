use axum::response::IntoResponse;
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