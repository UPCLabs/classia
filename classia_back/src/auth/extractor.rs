use axum::{extract::FromRequestParts, http::header::COOKIE};
use uuid::Uuid;

use crate::{error::AppError, state::ClassiaState, users::UserRole};

fn find_cookie<'a>(cookie_header: &'a str, expected_name: &str) -> Option<&'a str> {
    cookie_header
        .split(';')
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find_map(|(name, value)| (name == expected_name).then_some(value))
}

pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: UserRole,
}

impl FromRequestParts<ClassiaState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ClassiaState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_header = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(AppError::unauthorized)?;

        let token =
            find_cookie(cookie_header, "classia_access").ok_or_else(AppError::unauthorized)?;

        let claims = state.auth_service.validate_token(token)?;

        Ok(Self {
            id: claims.sub,
            role: claims.role,
        })
    }
}
