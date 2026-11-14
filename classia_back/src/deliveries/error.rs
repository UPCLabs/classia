use crate::{activities::ActivityError, error::AppError};
use axum::http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub enum DeliveryError {
    Activity(ActivityError),
    DeliveryNotFound,
    Forbidden,
    FileUnavailable,
    Database(sqlx::Error),
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
