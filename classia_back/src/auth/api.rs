use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::header::SET_COOKIE;
use serde_json::{Value, json};

use crate::{
    auth::{
        dto::{IdentityResponse, LoginRequest, TotpChallenge, TotpResponse},
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
        .route("/setup-mfa", post(setup_mfa))
        .route("/challenge-mfa", post(challenge_mfa))
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

async fn setup_mfa(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
) -> Result<Json<TotpResponse>, AppError> {
    let totp = state.auth_service.setup_mfa(user.id).await?;

    Ok(Json(TotpResponse {
        secret: totp.totp,
        url: totp.url,
    }))
}

async fn challenge_mfa(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Json(request): Json<TotpChallenge>,
) -> Result<Json<Value>, AppError> {
    let status = state
        .auth_service
        .challenge_mfa(user.id, &request.code)
        .await?;

    Ok(Json(json!({
        "verified": status
    })))
}
