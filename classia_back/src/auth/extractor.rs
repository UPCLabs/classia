use axum::{extract::FromRequestParts, http::header::COOKIE};
use uuid::Uuid;

use crate::{auth::claims::TokenPurpose, error::AppError, state::ClassiaState, users::UserRole};

pub(super) fn find_cookie<'a>(cookie_header: &'a str, expected_name: &str) -> Option<&'a str> {
    cookie_header
        .split(';')
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find_map(|(name, value)| (name == expected_name).then_some(value))
}

pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: UserRole,
}

impl AuthenticatedUser {
    /// Only a full session authorizes business routes; the partial MFA
    /// session is rejected here (SDS v2.2 §14.3).
    pub(super) async fn from_cookie_header(
        cookie_header: &str,
        state: &ClassiaState,
    ) -> Result<Self, AppError> {
        let token =
            find_cookie(cookie_header, "classia_access").ok_or_else(AppError::unauthorized)?;

        let claims = state.auth_service.validate_token(token)?;
        if claims.purpose != TokenPurpose::Access {
            return Err(AppError::invalid_token());
        }

        let user = state
            .user_service
            .find_active_user(claims.sub)
            .await?
            .ok_or_else(AppError::invalid_token)?;

        Ok(Self {
            id: user.id,
            role: user.role,
        })
    }
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

        Self::from_cookie_header(cookie_header, state).await
    }
}
