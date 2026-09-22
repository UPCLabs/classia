use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct BodyError {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "Authentication is required",
        )
    }

    pub fn invalid_token() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "INVALID_TOKEN",
            "The authentication token is invalid or expired",
        )
    }

    pub fn forbidden() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "You do not have permission to perform this action",
        )
    }

    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "An internal error occurred",
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = BodyError {
            code: self.code,
            message: self.message,
        };
        (self.status, Json(body)).into_response()
    }
}
