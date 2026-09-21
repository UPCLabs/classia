use crate::error::AppError;
use axum::http::StatusCode;

#[derive(Debug)]
pub enum CourseError {
    CourseNotFound,
    Database(sqlx::Error),
}

impl From<CourseError> for AppError {
    fn from(value: CourseError) -> Self {
        match value {
            CourseError::CourseNotFound => AppError {
                status: StatusCode::NOT_FOUND,
                code: "COURSE_NOT_FOUND",
                message: "Curso no encontrado".to_string(),
            },
            CourseError::Database(error) => {
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
