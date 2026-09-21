use crate::error::AppError;
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
                code: StatusCode::NOT_FOUND.as_str(),
                message: "User not found".to_string(),
            },
            UserError::Database(error) => {
                eprintln!("Database error: {}", error);
                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: StatusCode::NOT_FOUND.as_str(),
                    message: "Database error".to_string(),
                }
            }
        }
    }
}
