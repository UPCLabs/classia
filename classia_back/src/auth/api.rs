use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::header::SET_COOKIE;

use crate::{
    auth::{
        dto::{IdentityResponse, LoginRequest},
        extractor::AuthenticatedUser,
    },
    error::AppError,
    state::ClassiaState,
};

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/me", get(me))
        .route("/login", post(login))
        .route("/logout", post(logout))
}

async fn me(
    authenticated_user: AuthenticatedUser,
    State(state): State<ClassiaState>,
) -> Result<Json<IdentityResponse>, AppError> {
    let user = state
        .user_service
        .get_user_by_id(authenticated_user.id)
        .await?;

    Ok(Json(user.into()))
}

async fn login(
    State(state): State<ClassiaState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = state.auth_service.login(request).await?;

    let cookie = format!(
        "classia_access={token}; \
               Path=/; \
               HttpOnly; \
               Secure; \
               SameSite=Lax; \
               Max-Age=86400"
    );

    let cookie = HeaderValue::from_str(&cookie).map_err(|_| AppError::internal())?;

    Ok(([(SET_COOKIE, cookie)], StatusCode::NO_CONTENT))
}

async fn logout() -> Result<impl IntoResponse, AppError> {
    let cookie = HeaderValue::from_static(
        "classia_access=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0",
    );

    Ok(([(SET_COOKIE, cookie)], StatusCode::NO_CONTENT))
}
