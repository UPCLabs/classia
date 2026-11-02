use axum::http::StatusCode;
use tracing::error;

use crate::error::AppError;

#[derive(Debug)]
pub enum ActivityError {
    ActivityNotFound,
    CourseNotFound,
    Forbidden,
    CourseNotActive,
    AlreadyPublished,
    DueDateNotFuture,
    Database(sqlx::Error),
}

impl From<ActivityError> for AppError {
    fn from(value: ActivityError) -> Self {
        match value {
            ActivityError::ActivityNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "ACTIVITY_NOT_FOUND",
                "Activity not found",
            ),
            ActivityError::CourseNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "COURSE_NOT_FOUND",
                "Course not found",
            ),
            ActivityError::Forbidden => AppError::forbidden(),
            ActivityError::CourseNotActive => AppError::new(
                StatusCode::CONFLICT,
                "COURSE_NOT_ACTIVE",
                "The course is not active",
            ),
            ActivityError::AlreadyPublished => AppError::new(
                StatusCode::CONFLICT,
                "ACTIVITY_ALREADY_PUBLISHED",
                "The activity is already published",
            ),
            ActivityError::DueDateNotFuture => AppError::new(
                StatusCode::BAD_REQUEST,
                "DUE_DATE_NOT_FUTURE",
                "The due date must be in the future",
            ),
            ActivityError::Database(error) => database_error(error),
        }
    }
}

fn database_error(database: sqlx::Error) -> AppError {
    error!("Database error: {database}");
    AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "DATABASE_ERROR",
        "Error de base de datos",
    )
}
