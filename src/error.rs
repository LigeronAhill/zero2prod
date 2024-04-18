use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[derive(Debug)]
pub enum AppError {
    Custom(anyhow::Error),
    EmailAlreadyExists,
    DatabaseError,
    UserNotFound,
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Custom(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e}")).into_response()
            }
            AppError::EmailAlreadyExists => {
                (StatusCode::BAD_REQUEST, "Email already exists").into_response()
            }
            AppError::DatabaseError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
            AppError::UserNotFound => (StatusCode::BAD_REQUEST, "User not found").into_response(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Custom(err.into())
    }
}
pub type Result<T> = core::result::Result<T, AppError>;
