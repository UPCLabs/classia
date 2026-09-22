use crate::error::AppError;
use axum::http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub enum AuthError {
    JwtError(String),
    InternalError(String),
    InvalidCredentials,
    InactiveUser,
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::JwtError(reason) => {
                error!(error = &reason, "JWT error");
                AppError {
                    status: StatusCode::UNAUTHORIZED,
                    code: StatusCode::UNAUTHORIZED.as_str(),
                    message: reason,
                }
            }
            AuthError::InternalError(reason) => {
                error!(error = &reason, "Internal error");

                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: StatusCode::INTERNAL_SERVER_ERROR.as_str(),
                    message: reason,
                }
            }
            AuthError::InvalidCredentials => {
                error!("Invalid credentials");

                AppError {
                    status: StatusCode::UNAUTHORIZED,
                    code: StatusCode::UNAUTHORIZED.as_str(),
                    message: "Invalid credetianls".to_string(),
                }
            }
            AuthError::InactiveUser => AppError {
                status: StatusCode::BAD_REQUEST,
                code: StatusCode::BAD_REQUEST.as_str(),
                message: "Usuario no disponible".to_string(),
            },
        }
    }
}
