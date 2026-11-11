//! UC-005 end to end over HTTP: setup, activation and two-step login
//! (SDS v2.2 §14.3–14.4).

use argon2::{Argon2, password_hash::PasswordHasher};
use axum::{
    Router,
    body::Body,
    http::{
        Method, Request, Response, StatusCode,
        header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
    },
};
use classia_back::{JwtSettings, build_router};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use totp_rs::{Secret, Totp};
use tower::ServiceExt;

const JWT_SECRET: &str = "0123456789abcdef0123456789abcdef";
const MFA_KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const EMAIL: &str = "teacher@example.com";
const PASSWORD: &str = "correct-password";

fn app(pool: PgPool) -> Router {
    app_with_key(pool, MFA_KEY)
}

fn app_with_key(pool: PgPool, key: &str) -> Router {
    build_router(
        pool,
        JwtSettings {
            secret: JWT_SECRET.to_string(),
            issuer: "classia".to_string(),
            expiration: 3600,
            mfa_encryption_key: Some(key.to_string()),
        },
    )
    .expect("test router builds")
}

async fn insert_teacher(pool: &PgPool) {
    let hash = Argon2::default()
        .hash_password(PASSWORD.as_bytes())
        .expect("password hashing succeeds")
        .to_string();
    sqlx::query(
        "INSERT INTO users (name, email, password, role, status, token)
         VALUES ('Teacher', $1, $2, 'teacher', 'active', '')",
    )
    .bind(EMAIL)
    .bind(hash)
    .execute(pool)
    .await
    .expect("teacher is inserted");
}

async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    cookie: Option<&str>,
) -> Response<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    if let Some(cookie) = cookie {
        builder = builder.header(COOKIE, cookie);
    }
    app.clone()
        .oneshot(
            builder
                .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .expect("request builds"),
        )
        .await
        .expect("router handles request")
}

async fn json_body(response: Response<Body>) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("JSON body")
}

/// Returns `name=value` for a cookie that is being set (not cleared).
fn set_cookie(response: &Response<Body>, name: &str) -> Option<String> {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .find(|pair| pair.starts_with(&format!("{name}=")) && !pair.ends_with('='))
        .map(str::to_string)
}

async fn login(app: &Router) -> Response<Body> {
    send(
        app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "email": EMAIL, "password": PASSWORD })),
        None,
    )
    .await
}

fn totp(secret_base32: &str) -> Totp {
    totp_rs::Builder::new()
        .with_secret(Secret::try_from_base32(secret_base32).expect("base32 secret"))
        .build()
        .expect("valid TOTP")
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs()
}

/// Logs in, sets up MFA and activates it. Returns the base32 secret.
async fn activate_mfa(app: &Router) -> String {
    let access = set_cookie(&login(app).await, "classia_access").expect("full session");

    let setup = send(
        app,
        Method::POST,
        "/api/auth/setup-mfa",
        None,
        Some(&access),
    )
    .await;
    assert_eq!(setup.status(), StatusCode::OK);
    assert_eq!(setup.headers()["cache-control"], "no-store");
    let body = json_body(setup).await;
    let secret = body["secret"].as_str().expect("secret").to_string();
    let url = body["url"].as_str().expect("url");
    assert!(url.starts_with("otpauth://totp/Classia:"));
    assert!(url.contains("issuer=Classia"));

    // A code from the previous window is still accepted (±1 step), which keeps
    // the activation code distinct from the login code used later.
    let code = totp(&secret).generate(now() - 30).to_string();
    let activation = send(
        app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&access),
    )
    .await;
    assert_eq!(activation.status(), StatusCode::OK);
    assert_eq!(json_body(activation).await["verified"], true);

    secret
}

#[sqlx::test(migrations = "./migrations")]
async fn login_without_mfa_keeps_the_single_step_session(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool);

    let response = login(&app).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(set_cookie(&response, "classia_access").is_some());
    assert!(set_cookie(&response, "classia_mfa").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn pending_setup_does_not_require_a_second_factor(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool.clone());
    let access = set_cookie(&login(&app).await, "classia_access").expect("full session");

    let setup = send(
        &app,
        Method::POST,
        "/api/auth/setup-mfa",
        None,
        Some(&access),
    )
    .await;
    assert_eq!(setup.status(), StatusCode::OK);

    let status: String = sqlx::query_scalar("SELECT mfa_status FROM users WHERE email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("status");
    assert_eq!(status, "pending");
    assert_eq!(login(&app).await.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn secret_is_stored_encrypted_never_in_clear_text(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool.clone());
    let access = set_cookie(&login(&app).await, "classia_access").expect("full session");
    let setup = json_body(
        send(
            &app,
            Method::POST,
            "/api/auth/setup-mfa",
            None,
            Some(&access),
        )
        .await,
    )
    .await;
    let secret = setup["secret"].as_str().expect("secret");

    let (ciphertext, nonce, token): (Vec<u8>, Vec<u8>, String) = sqlx::query_as(
        "SELECT mfa_secret_ciphertext, mfa_nonce, token FROM users WHERE email = $1",
    )
    .bind(EMAIL)
    .fetch_one(&pool)
    .await
    .expect("stored secret");

    let raw = Secret::try_from_base32(secret).expect("base32").to_vec();
    assert_eq!(nonce.len(), 12);
    assert_ne!(ciphertext, raw);
    assert!(
        !ciphertext
            .windows(raw.len())
            .any(|window| window == raw.as_slice())
    );
    assert!(!token.contains(secret));
}

#[sqlx::test(migrations = "./migrations")]
async fn activation_rejects_wrong_code_and_keeps_factor_pending(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool);
    let access = set_cookie(&login(&app).await, "classia_access").expect("full session");
    send(
        &app,
        Method::POST,
        "/api/auth/setup-mfa",
        None,
        Some(&access),
    )
    .await;

    let wrong = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": "abc123" })),
        Some(&access),
    )
    .await;

    assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(wrong).await["code"], "MFA_CODE_INVALID");
    assert_eq!(login(&app).await.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn active_mfa_requires_the_code_before_a_full_session(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool);
    let secret = activate_mfa(&app).await;

    let first_step = login(&app).await;
    assert_eq!(first_step.status(), StatusCode::OK);
    assert!(set_cookie(&first_step, "classia_access").is_none());
    let headers: Vec<_> = first_step
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap())
        .collect();
    assert!(
        headers
            .iter()
            .any(|value| value.starts_with("classia_access=;") && value.contains("Max-Age=0"))
    );
    let partial_header = headers
        .iter()
        .find(|value| value.starts_with("classia_mfa="))
        .unwrap();
    for attribute in [
        "Path=/api/auth",
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Max-Age=300",
    ] {
        assert!(partial_header.contains(attribute));
    }
    let partial = set_cookie(&first_step, "classia_mfa").expect("partial session");
    assert_eq!(json_body(first_step).await["mfa_required"], true);

    // The partial session never authorizes business routes.
    let me = send(&app, Method::GET, "/api/auth/me", None, Some(&partial)).await;
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);
    let renamed = partial.replacen("classia_mfa", "classia_access", 1);
    let forged = send(&app, Method::GET, "/api/courses", None, Some(&renamed)).await;
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);

    let code = totp(&secret).generate(now()).to_string();
    let second_step = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&partial),
    )
    .await;
    assert_eq!(second_step.status(), StatusCode::OK);
    let access = set_cookie(&second_step, "classia_access").expect("full session after code");

    let me = send(&app, Method::GET, "/api/auth/me", None, Some(&access)).await;
    assert_eq!(me.status(), StatusCode::OK);
    assert_eq!(json_body(me).await["email"], EMAIL);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_code_cannot_be_reused(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool);
    let secret = activate_mfa(&app).await;
    let code = totp(&secret).generate(now()).to_string();

    let partial = set_cookie(&login(&app).await, "classia_mfa").expect("partial session");
    let first = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&partial),
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);

    let partial = set_cookie(&login(&app).await, "classia_mfa").expect("partial session");
    let replay = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&partial),
    )
    .await;
    assert_eq!(replay.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(replay).await["code"], "MFA_CODE_REUSED");
}

#[sqlx::test(migrations = "./migrations")]
async fn five_failures_lock_the_verification(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool);
    let secret = activate_mfa(&app).await;
    let partial = set_cookie(&login(&app).await, "classia_mfa").expect("partial session");

    for attempt in 1..=5 {
        let response = send(
            &app,
            Method::POST,
            "/api/auth/challenge-mfa",
            Some(json!({ "code": "invalid" })),
            Some(&partial),
        )
        .await;
        let expected = if attempt < 5 {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::TOO_MANY_REQUESTS
        };
        assert_eq!(response.status(), expected, "attempt {attempt}");
        if attempt == 5 {
            assert_eq!(response.headers()["retry-after"], "300");
        }
    }

    // Even a valid code is refused while locked; a new login does not bypass it.
    let partial = set_cookie(&login(&app).await, "classia_mfa").expect("partial session");
    let code = totp(&secret).generate(now()).to_string();
    let locked = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&partial),
    )
    .await;
    assert_eq!(locked.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after: u64 = locked.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=300).contains(&retry_after));
    assert_eq!(json_body(locked).await["code"], "MFA_LOCKED");
}

#[sqlx::test(migrations = "./migrations")]
async fn setup_is_refused_once_mfa_is_active(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool);
    let secret = activate_mfa(&app).await;

    let partial = set_cookie(&login(&app).await, "classia_mfa").expect("partial session");
    let code = totp(&secret).generate(now()).to_string();
    let response = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&partial),
    )
    .await;
    let access = set_cookie(&response, "classia_access").expect("full session");

    let again = send(
        &app,
        Method::POST,
        "/api/auth/setup-mfa",
        None,
        Some(&access),
    )
    .await;
    assert_eq!(again.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(again).await["code"], "MFA_ALREADY_ENABLED");
}

#[sqlx::test(migrations = "./migrations")]
async fn challenge_without_any_session_is_rejected(pool: PgPool) {
    let app = app(pool);

    let response = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": "123456" })),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn pending_setup_cannot_bypass_a_lock_and_recovers_after_expiry(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool.clone());
    let access = set_cookie(&login(&app).await, "classia_access").unwrap();
    let setup = json_body(
        send(
            &app,
            Method::POST,
            "/api/auth/setup-mfa",
            None,
            Some(&access),
        )
        .await,
    )
    .await;
    let secret = setup["secret"].as_str().unwrap();
    for _ in 0..5 {
        send(
            &app,
            Method::POST,
            "/api/auth/challenge-mfa",
            Some(json!({ "code": "invalid" })),
            Some(&access),
        )
        .await;
    }
    let setup = send(
        &app,
        Method::POST,
        "/api/auth/setup-mfa",
        None,
        Some(&access),
    )
    .await;
    assert_eq!(setup.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(setup.headers().contains_key("retry-after"));

    // Move only the persisted deadline, not the process clock or real time.
    sqlx::query("UPDATE users SET mfa_locked_until = now() - interval '1 second' WHERE email = $1")
        .bind(EMAIL)
        .execute(&pool)
        .await
        .unwrap();
    let response = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": totp(secret).generate(now()).to_string() })),
        Some(&access),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let state: (String, i32, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT mfa_status, mfa_failed_attempts, mfa_locked_until FROM users WHERE email = $1",
    )
    .bind(EMAIL)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state, ("active".to_string(), 0, None));
}

#[sqlx::test(migrations = "./migrations")]
async fn replacing_pending_setup_invalidates_the_previous_secret(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool);
    let access = set_cookie(&login(&app).await, "classia_access").unwrap();
    let first = json_body(
        send(
            &app,
            Method::POST,
            "/api/auth/setup-mfa",
            None,
            Some(&access),
        )
        .await,
    )
    .await;
    let second = json_body(
        send(
            &app,
            Method::POST,
            "/api/auth/setup-mfa",
            None,
            Some(&access),
        )
        .await,
    )
    .await;
    let old = totp(first["secret"].as_str().unwrap());
    let new = totp(second["secret"].as_str().unwrap());
    assert_ne!(first["secret"], second["secret"]);
    // Avoid a coincidental six-digit collision between independent secrets.
    let timestamp = now();
    let new_codes: Vec<_> = [timestamp - 30, timestamp, timestamp + 30]
        .map(|time| new.generate(time).to_string())
        .to_vec();
    let old_code = [timestamp - 30, timestamp, timestamp + 30]
        .into_iter()
        .map(|time| old.generate(time).to_string())
        .find(|code| !new_codes.contains(code))
        .unwrap();
    let rejected = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": old_code })),
        Some(&access),
    )
    .await;
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    let accepted = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": new.generate(now()).to_string() })),
        Some(&access),
    )
    .await;
    assert_eq!(accepted.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_challenges_accept_a_code_exactly_once(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool.clone());
    let secret = activate_mfa(&app).await;
    let partial = set_cookie(&login(&app).await, "classia_mfa").unwrap();
    let body = json!({ "code": totp(&secret).generate(now()).to_string() });
    let (first, second) = tokio::join!(
        send(
            &app,
            Method::POST,
            "/api/auth/challenge-mfa",
            Some(body.clone()),
            Some(&partial)
        ),
        send(
            &app,
            Method::POST,
            "/api/auth/challenge-mfa",
            Some(body),
            Some(&partial)
        ),
    );
    let (success, denied) = if first.status() == StatusCode::OK {
        (first, second)
    } else {
        (second, first)
    };
    assert_eq!(success.status(), StatusCode::OK);
    assert!(set_cookie(&success, "classia_access").is_some());
    assert_eq!(denied.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(denied).await["code"], "MFA_CODE_REUSED");
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE action = 'mfa.challenge' AND result = 'success'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn expired_or_wrong_purpose_partial_sessions_never_authorize_a_challenge(pool: PgPool) {
    insert_teacher(&pool).await;
    let id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .unwrap();
    let app = app(pool);
    activate_mfa(&app).await;
    for (purpose, expires) in [
        (Some("mfa_pending"), now() as i64 - 1),
        (Some("access"), now() as i64 + 300),
        (None, now() as i64 + 300),
    ] {
        let mut claims = json!({ "sub": id, "role": "Teacher", "iat": now() as i64 - 300,
            "exp": expires, "iss": "classia" });
        if let Some(purpose) = purpose {
            claims["purpose"] = json!(purpose);
        }
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )
        .unwrap();
        let response = send(
            &app,
            Method::POST,
            "/api/auth/challenge-mfa",
            Some(json!({ "code": "123456" })),
            Some(&format!("classia_mfa={token}")),
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(set_cookie(&response, "classia_access").is_none());
        assert_eq!(json_body(response).await["code"], "MFA_SESSION_EXPIRED");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn wrong_encryption_key_fails_closed_without_exposing_crypto_details(pool: PgPool) {
    insert_teacher(&pool).await;
    let correct_app = app(pool.clone());
    let secret = activate_mfa(&correct_app).await;
    let partial = set_cookie(&login(&correct_app).await, "classia_mfa").unwrap();
    let wrong_app = app_with_key(pool, &"ff".repeat(32));
    let response = send(
        &wrong_app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": totp(&secret).generate(now()).to_string() })),
        Some(&partial),
    )
    .await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(set_cookie(&response, "classia_access").is_none());
    let body = json_body(response).await;
    assert_eq!(body["code"], "INTERNAL_ERROR");
    assert_eq!(
        body["message"],
        "No se pudo completar la operación. Inténtalo de nuevo."
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn setup_activation_and_failures_are_audited_without_secrets(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool.clone());
    let secret = activate_mfa(&app).await;
    let partial = set_cookie(&login(&app).await, "classia_mfa").unwrap();
    send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": "invalid" })),
        Some(&partial),
    )
    .await;
    let code = totp(&secret).generate(now()).to_string();
    send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&partial),
    )
    .await;
    send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(json!({ "code": code })),
        Some(&partial),
    )
    .await;

    let events: Vec<(String, String, Value)> = sqlx::query_as(
        "SELECT action, result::text, details FROM audit_events
         WHERE target_type = 'user' AND actor_user_id = target_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(events.len(), 5);
    for (action, result) in [
        ("mfa.setup_started", "success"),
        ("mfa.activated", "success"),
        ("mfa.challenge", "success"),
        ("mfa.challenge", "failure"),
        ("mfa.challenge", "denied"),
    ] {
        assert!(
            events
                .iter()
                .any(|(actual_action, actual_result, _)| actual_action == action
                    && actual_result == result)
        );
    }
    for (_, _, details) in events {
        assert!(!details.to_string().contains(&secret));
        assert!(!details.to_string().contains(&code));
        assert!(
            details
                .as_object()
                .unwrap()
                .keys()
                .all(|key| key == "reason")
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn failed_audit_rolls_back_activation_and_the_consumed_step(pool: PgPool) {
    insert_teacher(&pool).await;
    let app = app(pool.clone());
    let access = set_cookie(&login(&app).await, "classia_access").unwrap();
    let setup = json_body(
        send(
            &app,
            Method::POST,
            "/api/auth/setup-mfa",
            None,
            Some(&access),
        )
        .await,
    )
    .await;
    sqlx::raw_sql(
        "CREATE FUNCTION reject_mfa_audit() RETURNS trigger LANGUAGE plpgsql AS $$
        BEGIN RAISE EXCEPTION 'simulated audit failure'; END $$;
        CREATE TRIGGER reject_mfa_audit BEFORE INSERT ON audit_events
        FOR EACH ROW EXECUTE FUNCTION reject_mfa_audit();",
    )
    .execute(&pool)
    .await
    .unwrap();
    let response = send(
        &app,
        Method::POST,
        "/api/auth/challenge-mfa",
        Some(
            json!({ "code": totp(setup["secret"].as_str().unwrap()).generate(now()).to_string() }),
        ),
        Some(&access),
    )
    .await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let state: (String, Option<i64>) =
        sqlx::query_as("SELECT mfa_status, mfa_last_step FROM users WHERE email = $1")
            .bind(EMAIL)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(state, ("pending".to_string(), None));
    assert_eq!(login(&app).await.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = false)]
async fn migration_upgrades_existing_users_without_rewriting_applied_history(pool: PgPool) {
    let mut previous = sqlx::migrate!("./migrations");
    previous.migrations = std::borrow::Cow::Owned(
        previous
            .iter()
            .filter(|migration| migration.version < 20261111190000)
            .cloned()
            .collect(),
    );
    previous.run(&pool).await.unwrap();
    insert_teacher(&pool).await;
    let before: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let after: Vec<(i64, Vec<u8>)> = sqlx::query_as(
        "SELECT version, checksum FROM _sqlx_migrations WHERE version < 20261111190000 ORDER BY version")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(before, after);
    let state: (String, String, String, Option<Vec<u8>>) = sqlx::query_as(
        "SELECT email, token, mfa_status, mfa_secret_ciphertext FROM users WHERE email = $1",
    )
    .bind(EMAIL)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        state,
        (
            EMAIL.to_string(),
            String::new(),
            "disabled".to_string(),
            None
        )
    );
    assert_eq!(login(&app(pool)).await.status(), StatusCode::NO_CONTENT);
}
