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
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "0123456789abcdef0123456789abcdef";

fn app(pool: PgPool) -> Router {
    build_router(
        pool,
        JwtSettings {
            secret: JWT_SECRET.to_string(),
            issuer: "classia".to_string(),
            expiration: 3600,
        },
    )
    .expect("test router builds")
}

fn password_hash(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes())
        .expect("password hashing succeeds")
        .to_string()
}

async fn insert_user(pool: &PgPool, email: &str, password: &str, role: &str, status: &str) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (name, email, password, role, status, token)
        VALUES ('Test User', $1, $2, $3::user_role, $4, '')
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(password_hash(password))
    .bind(role)
    .bind(status)
    .fetch_one(pool)
    .await
    .expect("test user is inserted")
}

async fn request(
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
                .body(match body {
                    Some(value) => Body::from(value.to_string()),
                    None => Body::empty(),
                })
                .expect("request builds"),
        )
        .await
        .expect("router handles request")
}

async fn response_json(response: Response<Body>) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body can be read")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("response body is JSON")
}

async fn login_cookie(app: &Router, email: &str, password: &str) -> String {
    let response = request(
        app,
        Method::POST,
        "/auth/login",
        Some(json!({ "email": email, "password": password })),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    response
        .headers()
        .get(SET_COOKIE)
        .expect("login returns a cookie")
        .to_str()
        .expect("cookie header is valid")
        .split(';')
        .next()
        .expect("cookie has a name and value")
        .to_string()
}

#[sqlx::test(migrations = "./migrations")]
async fn login_accepts_valid_credentials_and_rejects_invalid_or_inactive_users(pool: PgPool) {
    insert_user(
        &pool,
        "active@example.com",
        "correct-password",
        "student",
        "active",
    )
    .await;
    insert_user(
        &pool,
        "inactive@example.com",
        "correct-password",
        "student",
        "inactive",
    )
    .await;
    let app = app(pool);

    let valid = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({
            "email": "active@example.com",
            "password": "correct-password"
        })),
        None,
    )
    .await;
    assert_eq!(valid.status(), StatusCode::NO_CONTENT);
    assert!(valid.headers().contains_key(SET_COOKIE));

    let invalid = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({
            "email": "active@example.com",
            "password": "wrong-password"
        })),
        None,
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);

    let inactive = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({
            "email": "inactive@example.com",
            "password": "correct-password"
        })),
        None,
    )
    .await;
    assert_eq!(inactive.status(), StatusCode::BAD_REQUEST);

    let hidden_inactive_status = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({
            "email": "inactive@example.com",
            "password": "wrong-password"
        })),
        None,
    )
    .await;
    assert_eq!(hidden_inactive_status.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn user_creation_enforces_roles_and_duplicate_email(pool: PgPool) {
    insert_user(
        &pool,
        "admin@example.com",
        "admin-password",
        "admin",
        "active",
    )
    .await;
    insert_user(
        &pool,
        "student@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let app = app(pool.clone());
    let admin_cookie = login_cookie(&app, "admin@example.com", "admin-password").await;
    let student_cookie = login_cookie(&app, "student@example.com", "student-password").await;
    let payload = json!({
        "name": "New Teacher",
        "email": "teacher@example.com",
        "password": "teacher-password",
        "role": "Teacher"
    });

    let created = request(
        &app,
        Method::POST,
        "/users/create",
        Some(payload.clone()),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);

    let stored_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("teacher@example.com")
        .fetch_one(&pool)
        .await
        .expect("created user can be queried");
    assert_eq!(stored_count, 1);

    let duplicate = request(
        &app,
        Method::POST,
        "/users/create",
        Some(payload.clone()),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let forbidden = request(
        &app,
        Method::POST,
        "/users/create",
        Some(json!({
            "name": "Another User",
            "email": "another@example.com",
            "password": "another-password",
            "role": "Student"
        })),
        Some(&student_cookie),
    )
    .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn password_change_replaces_the_login_credential(pool: PgPool) {
    insert_user(
        &pool,
        "user@example.com",
        "old-password",
        "student",
        "active",
    )
    .await;
    let app = app(pool);
    let cookie = login_cookie(&app, "user@example.com", "old-password").await;

    let changed = request(
        &app,
        Method::PATCH,
        "/users/change-password",
        Some(json!({ "new_password": "new-password" })),
        Some(&cookie),
    )
    .await;
    assert_eq!(changed.status(), StatusCode::NO_CONTENT);

    let old_login = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({ "email": "user@example.com", "password": "old-password" })),
        None,
    )
    .await;
    assert_eq!(old_login.status(), StatusCode::UNAUTHORIZED);

    let new_login = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({ "email": "user@example.com", "password": "new-password" })),
        None,
    )
    .await;
    assert_eq!(new_login.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn courses_can_be_created_and_queried_through_existing_routes(pool: PgPool) {
    let teacher_id = insert_user(
        &pool,
        "teacher@example.com",
        "teacher-password",
        "teacher",
        "active",
    )
    .await;
    let app = app(pool);
    let teacher_cookie = login_cookie(&app, "teacher@example.com", "teacher-password").await;

    let created = request(
        &app,
        Method::POST,
        "/courses/create",
        Some(json!({
            "name": "Software Engineering",
            "code": "IS212",
            "teacher_id": teacher_id,
            "password": "course-password",
            "status": "active",
            "quantity": 25
        })),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created = response_json(created).await;
    let course_id = created["id"].as_str().expect("course id is present");

    for uri in [
        format!("/courses/ById/{course_id}"),
        "/courses/getCourseByCode/IS212".to_string(),
        format!("/courses/getTeachersCourse/{teacher_id}"),
    ] {
        let response = request(&app, Method::GET, &uri, None, Some(&teacher_cookie)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["code"], "IS212");
    }

    let missing = request(
        &app,
        Method::GET,
        &format!("/courses/ById/{}", Uuid::now_v7()),
        None,
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}
