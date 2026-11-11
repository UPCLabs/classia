use axum::{
    Json, Router,
    extract::State,
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{COOKIE, RETRY_AFTER},
    },
    response::{AppendHeaders, IntoResponse, Response},
    routing::{get, post},
};
use reqwest::header::{CACHE_CONTROL, SET_COOKIE};
use serde_json::json;

use crate::{
    auth::{
        claims::TokenPurpose,
        dto::{IdentityResponse, LoginRequest, TotpChallenge, TotpResponse},
        error::AuthError,
        extractor::{AuthenticatedUser, find_cookie},
        service::{ChallengeMode, LoginOutcome, MFA_SESSION_SECS},
    },
    error::AppError,
    state::ClassiaState,
};

const ACCESS_COOKIE: &str = "classia_access";
const MFA_COOKIE: &str = "classia_mfa";
const ACCESS_MAX_AGE: i64 = 86400;

/// Keeps the shared JSON error contract while exposing the MFA lock duration.
struct AuthApiError {
    error: AppError,
    retry_after_secs: Option<i64>,
}

impl From<AppError> for AuthApiError {
    fn from(error: AppError) -> Self {
        Self {
            error,
            retry_after_secs: None,
        }
    }
}

impl From<AuthError> for AuthApiError {
    fn from(error: AuthError) -> Self {
        let retry_after_secs = match &error {
            AuthError::MfaLocked { retry_after_secs } => Some(*retry_after_secs),
            _ => None,
        };
        Self {
            error: error.into(),
            retry_after_secs,
        }
    }
}

impl IntoResponse for AuthApiError {
    fn into_response(self) -> Response {
        let mut response = self.error.into_response();
        if let Some(seconds) = self.retry_after_secs
            && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(RETRY_AFTER, value);
        }
        response
    }
}

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/me", get(me))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/setup-mfa", post(setup_mfa))
        .route("/challenge-mfa", post(challenge_mfa))
}

fn access_cookie(token: &str) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(&format!(
        "{ACCESS_COOKIE}={token}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={ACCESS_MAX_AGE}"
    ))
    .map_err(|_| AppError::internal())
}

/// The partial session is only valid for the authentication routes.
fn mfa_cookie(token: &str) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(&format!(
        "{MFA_COOKIE}={token}; Path=/api/auth; HttpOnly; Secure; SameSite=Lax; Max-Age={MFA_SESSION_SECS}"
    ))
    .map_err(|_| AppError::internal())
}

fn clear_access_cookie() -> HeaderValue {
    HeaderValue::from_static("classia_access=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0")
}

fn clear_mfa_cookie() -> HeaderValue {
    HeaderValue::from_static(
        "classia_mfa=; Path=/api/auth; HttpOnly; Secure; SameSite=Lax; Max-Age=0",
    )
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
) -> Result<Response, AppError> {
    match state.auth_service.login(request).await? {
        LoginOutcome::Authenticated(token) => Ok((
            AppendHeaders([
                (SET_COOKIE, access_cookie(&token)?),
                (SET_COOKIE, clear_mfa_cookie()),
            ]),
            StatusCode::NO_CONTENT,
        )
            .into_response()),
        LoginOutcome::MfaRequired(token) => Ok((
            AppendHeaders([
                (SET_COOKIE, clear_access_cookie()),
                (SET_COOKIE, mfa_cookie(&token)?),
            ]),
            Json(json!({ "mfa_required": true })),
        )
            .into_response()),
    }
}

async fn logout() -> Result<impl IntoResponse, AppError> {
    Ok((
        AppendHeaders([
            (SET_COOKIE, clear_access_cookie()),
            (SET_COOKIE, clear_mfa_cookie()),
        ]),
        StatusCode::NO_CONTENT,
    ))
}

async fn setup_mfa(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
) -> Result<Response, AuthApiError> {
    let setup = state.auth_service.setup_mfa(user.id).await?;

    Ok((
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(TotpResponse {
            secret: setup.secret,
            url: setup.url,
        }),
    )
        .into_response())
}

/// Completes a two-step login (partial session) or activates a pending setup
/// (full session). The partial session takes precedence when both are present.
async fn challenge_mfa(
    State(state): State<ClassiaState>,
    headers: HeaderMap,
    Json(request): Json<TotpChallenge>,
) -> Result<Response, AuthApiError> {
    let cookies = headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if let Some(token) = find_cookie(cookies, MFA_COOKIE) {
        let claims = state
            .auth_service
            .validate_token(token)
            .ok()
            .filter(|claims| claims.purpose == TokenPurpose::MfaPending)
            .ok_or_else(|| AppError::from(crate::auth::error::AuthError::MfaSessionExpired))?;

        let access_token = state
            .auth_service
            .challenge_mfa(claims.sub, &request.code, ChallengeMode::Login)
            .await?
            .ok_or_else(AppError::internal)?;

        return Ok((
            AppendHeaders([
                (SET_COOKIE, access_cookie(&access_token)?),
                (SET_COOKIE, clear_mfa_cookie()),
            ]),
            Json(json!({ "verified": true })),
        )
            .into_response());
    }

    let user = AuthenticatedUser::from_cookie_header(cookies, &state).await?;
    state
        .auth_service
        .challenge_mfa(user.id, &request.code, ChallengeMode::Activation)
        .await?;

    Ok(Json(json!({ "verified": true })).into_response())
}
