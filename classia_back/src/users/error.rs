use crate::error::AppError;
use axum::http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub enum UserError {
    UserNotFound,
    EmailAlreadyExists,
    InvalidCredentials,
    Forbidden(String),
    ValidationError(String),
    InternalError(String),
    Database(sqlx::Error),
}

impl From<UserError> for AppError {
    fn from(value: UserError) -> Self {
        match value {
            UserError::UserNotFound => AppError {
                status: StatusCode::NOT_FOUND,
                code: StatusCode::NOT_FOUND.as_str(),
                message: "User not found".to_string(),
            },
            UserError::Database(error) => {
                error!("Database error: {}", error);
                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: StatusCode::INTERNAL_SERVER_ERROR.as_str(),
                    message: "Database error".to_string(),
                }
            }
            UserError::EmailAlreadyExists => AppError {
                status: StatusCode::CONFLICT,
                code: "EMAIL_ALREADY_EXISTS",
                message: "El correo ya existe".to_string(),
            },
            UserError::InvalidCredentials => AppError {
                status: StatusCode::UNAUTHORIZED,
                code: "INVALID_CREDENTIALS",
                message: "Contraseña incorrecta ".to_string(),
            },
            UserError::Forbidden(message) => AppError {
                status: StatusCode::FORBIDDEN,
                code: "FORBIDDEN",
                message,
            },
            UserError::ValidationError(message) => AppError {
                status: StatusCode::BAD_REQUEST,
                code: "VALIDATION_ERROR",
                message,
            },
            UserError::InternalError(detalle) => {
                error!("InternalError: {}", detalle);
                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR",
                    message: "Error interno al procesar la solicitud".into(),
                }
            }
        }
    }
}
