use crate::{activities::ActivityError, error::AppError};
use axum::http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub enum DeliveryError {
    Activity(ActivityError),
    DeliveryNotFound,
    Forbidden,
    FileUnavailable,
    Validation(String),
    InvalidFileType,
    FileTooLarge,
    DueDatePassed,
    AlreadyExists,
    VersionConflict,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for DeliveryError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

impl From<ActivityError> for DeliveryError {
    fn from(value: ActivityError) -> Self {
        Self::Activity(value)
    }
}

impl From<DeliveryError> for AppError {
    fn from(value: DeliveryError) -> Self {
        match value {
            DeliveryError::Activity(e) => e.into(),
            DeliveryError::DeliveryNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "DELIVERY_NOT_FOUND",
                "Entrega no encontrada",
            ),
            DeliveryError::Forbidden => AppError::forbidden(),
            DeliveryError::FileUnavailable => AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "FILE_UNAVAILABLE",
                "El archivo de la entrega no está disponible",
            ),
            DeliveryError::Validation(message) => {
                AppError::new(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message)
            }
            DeliveryError::InvalidFileType => AppError::new(
                StatusCode::BAD_REQUEST,
                "INVALID_FILE_TYPE",
                "El archivo debe estar en formato Docx",
            ),
            DeliveryError::FileTooLarge => AppError::new(
                StatusCode::BAD_REQUEST,
                "FILE_TOO_LARGE",
                "El archivo supera el tamaño máximo de 20 MB",
            ),
            DeliveryError::DueDatePassed => AppError::new(
                StatusCode::BAD_REQUEST,
                "DUE_DATE_PASSED",
                "El plazo de entrega ya venció",
            ),
            DeliveryError::AlreadyExists => AppError::new(
                StatusCode::CONFLICT,
                "DELIVERY_ALREADY_EXISTS",
                "Ya registraste una entrega para esta actividad",
            ),
            DeliveryError::VersionConflict => AppError::new(
                StatusCode::CONFLICT,
                "DELIVERY_VERSION_CONFLICT",
                "La entrega cambió; recarga para ver la versión vigente",
            ),
            DeliveryError::Database(e) => {
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
