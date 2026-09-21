use crate::app_error::AppError;
use axum::http::StatusCode;

#[derive(Debug)]
pub enum UserError {
    UserNotFound,
    EmailAlreadyExists,
    ValidationError(String),
    InternalError(String),
    Database(sqlx::Error),
}

impl From<UserError> for AppError {
    fn from(value: UserError) -> Self {
        match value {
            UserError::UserNotFound => AppError {
                status: StatusCode::NOT_FOUND,
                code: "USER_NOT_FOUND",
                message: "User not found".to_string(),
            },
            UserError::Database(error) => {
                eprintln!("Database error: {}", error);
                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "DATABASE_ERROR",
                    message: "Database error".to_string(),
                }
            }
            UserError::EmailAlreadyExists => AppError {
                status: StatusCode::CONFLICT,
                code: "EMAIL_ALREADY_EXISTS",
                message: "Email already exists".to_string(),
            },
            UserError::ValidationError(message) => AppError {
                status: StatusCode::BAD_REQUEST,
                code: "VALIDATION_ERROR",
                message,
            },
            UserError::InternalError(detalle) => {
                eprintln!("InternalError: {}", detalle);
                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR",
                    message: "Error interno al procesar la solicitud".into(),
                }
            }
        }
    }
}
