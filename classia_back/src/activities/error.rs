use crate::{courses::CourseError, error::AppError};
use axum::http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub enum ActivityError {
    ActivityNotFound,
    Forbidden,
    CourseNotFound,
    CourseNotActive,
    Validation(String),
    DueDateNotFuture,
    NoChanges,
    AlreadyPublished,
    Database(sqlx::Error),
}

impl From<CourseError> for ActivityError {
    fn from(value: CourseError) -> Self {
        match value {
            CourseError::CourseNotFound => Self::CourseNotFound,
            CourseError::Forbidden => Self::Forbidden,
            CourseError::Database(e) => Self::Database(e),
            _ => Self::CourseNotFound,
        }
    }
}

impl From<ActivityError> for AppError {
    fn from(value: ActivityError) -> Self {
        match value {
            ActivityError::ActivityNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "ACTIVITY_NOT_FOUND",
                "Actividad no encontrada",
            ),
            ActivityError::CourseNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "COURSE_NOT_FOUND",
                "Curso no encontrado",
            ),
            ActivityError::CourseNotActive => AppError::new(
                StatusCode::CONFLICT,
                "COURSE_NOT_ACTIVE",
                "El curso no está activo",
            ),
            ActivityError::Forbidden => AppError::forbidden(),
            ActivityError::Validation(message) => {
                AppError::new(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message)
            }
            ActivityError::DueDateNotFuture => AppError::new(
                StatusCode::BAD_REQUEST,
                "DUE_DATE_NOT_FUTURE",
                "La fecha de entrega debe ser futura",
            ),
            ActivityError::NoChanges => AppError::new(
                StatusCode::BAD_REQUEST,
                "NO_CHANGES",
                "No hay cambios para aplicar",
            ),
            ActivityError::AlreadyPublished => AppError::new(
                StatusCode::CONFLICT,
                "ACTIVITY_ALREADY_PUBLISHED",
                "La actividad ya está publicada",
            ),
            ActivityError::Database(e) => {
                error!("Database error: {e}");
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Error de base de datos",
                )
            }
        }
    }
}
