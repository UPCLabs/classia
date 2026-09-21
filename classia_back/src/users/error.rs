use crate::app_error::AppError;
use axum::http::StatusCode;

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
        }
    }
}
