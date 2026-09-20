use axum::http::StatusCode;
use crate::app_error::AppError;

#[derive(Debug)]
pub enum UserError {
    UserNotFound,
    Database(sqlx::Error),
}

impl From<UserError> for AppError {
    fn from(value: UserError) -> Self {
        match value {
            UserError::UserNotFound => AppError {
                status: StatusCode::NOT_FOUND,
                code: "USER_NOT_FOUND",
                message: "Usuario no encontrado".to_string(),
            },
            UserError::Database(error) => {
                eprintln!("Database error: {}", error);
                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "DATABASE_ERROR",
                    message: "Error en la base de datos".to_string(),
                }
            }
        }
    }
}