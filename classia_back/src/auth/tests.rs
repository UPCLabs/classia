use chrono::Utc;
use uuid::Uuid;

use super::{
    error::AuthError,
    extractor::find_cookie,
    jwt::{JwtConfig, JwtService},
};
use crate::{
    error::AppError,
    users::{User, UserRole},
};

const SECRET: &str = "0123456789abcdef0123456789abcdef";

fn user(role: UserRole) -> User {
    let now = Utc::now();
    User {
        id: Uuid::now_v7(),
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "unused".to_string(),
        role,
        status: "active".to_string(),
        token: String::new(),
        created_at: now,
        updated_at: now,
    }
}

fn service(secret: &str, issuer: &str, expiration: i64) -> JwtService {
    JwtService::new(JwtConfig {
        secret: secret.to_string(),
        issuer: issuer.to_string(),
        expiration,
    })
    .expect("valid JWT configuration")
}

#[test]
fn rejects_short_jwt_secret() {
    let result = JwtService::new(JwtConfig {
        secret: "too-short".to_string(),
        issuer: "classia".to_string(),
        expiration: 60,
    });

    assert!(
        matches!(result, Err(AuthError::InternalError(message)) if message == "Secret is too short")
    );
}

#[test]
fn jwt_round_trip_preserves_identity_and_role() {
    let jwt = service(SECRET, "classia", 300);
    let user = user(UserRole::Teacher);

    let token = jwt.generate(&user).expect("token generation succeeds");
    let claims = jwt.validate(&token).expect("generated token is valid");

    assert_eq!(claims.sub, user.id);
    assert_eq!(claims.role, UserRole::Teacher);
    assert_eq!(claims.iss, "classia");
    assert_eq!(claims.exp - claims.iat, 300);
}

#[test]
fn rejects_token_signed_with_another_secret() {
    let token = service(SECRET, "classia", 300)
        .generate(&user(UserRole::Student))
        .expect("token generation succeeds");
    let other = service("fedcba9876543210fedcba9876543210", "classia", 300);

    assert!(
        matches!(other.validate(&token), Err(AuthError::JwtError(message)) if message == "Token invalid")
    );
}

#[test]
fn rejects_expired_token() {
    let jwt = service(SECRET, "classia", -120);
    let token = jwt
        .generate(&user(UserRole::Admin))
        .expect("token generation succeeds");

    assert!(
        matches!(jwt.validate(&token), Err(AuthError::JwtError(message)) if message == "Token expired")
    );
}

#[test]
fn cookie_lookup_matches_exact_name() {
    let cookies = "theme=dark; classia_access=abc.def==; classia_access_backup=other";

    assert_eq!(find_cookie(cookies, "classia_access"), Some("abc.def=="));
    assert_eq!(find_cookie(cookies, "missing"), None);
}

#[test]
fn auth_errors_map_to_expected_statuses() {
    let invalid: AppError = AuthError::InvalidCredentials.into();
    let inactive: AppError = AuthError::InactiveUser.into();
    let internal: AppError = AuthError::InternalError("failure".to_string()).into();

    assert_eq!(invalid.status, axum::http::StatusCode::UNAUTHORIZED);
    assert_eq!(inactive.status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(
        internal.status,
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}
