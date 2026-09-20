use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug)]
pub enum UserError {
    UserNotFound,
    Database(sqlx::Error),
}

#[derive(Debug, Serialize)]
struct BodyError {
    code: &'static str,
    message: String,
} 

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            UserError::UserNotFound => (
                StatusCode::NOT_FOUND,
                "USER_NOT_FOUND",
                "Usuario no encontrado".into(),
            ),
            UserError::Database(error) => {
                eprintln!("Database error: {}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Error en la base de datos".into(),
                )
            }
        };

        let body = BodyError { code, message };
        (status, Json(body)).into_response()
    }
}
