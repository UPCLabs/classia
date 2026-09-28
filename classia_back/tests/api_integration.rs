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
        "/api/auth/login",
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
        "/api/auth/login",
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
        "/api/auth/login",
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
        "/api/auth/login",
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
        "/api/auth/login",
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
        "email": "Teacher@Example.com",
        "password": "teacher-password",
        "role": "Teacher"
    });

    let created = request(
        &app,
        Method::POST,
        "/api/users/create",
        Some(payload.clone()),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let created = response_json(created).await;
    assert_eq!(created["email"], "teacher@example.com");
    assert_eq!(created["role"], "Teacher");
    assert_eq!(created["status"], "active");
    assert!(created.get("password").is_none());
    assert!(created.get("token").is_none());

    let stored_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("teacher@example.com")
        .fetch_one(&pool)
        .await
        .expect("created user can be queried");
    assert_eq!(stored_count, 1);

    let duplicate = request(
        &app,
        Method::POST,
        "/api/users/create",
        Some(payload.clone()),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let forbidden = request(
        &app,
        Method::POST,
        "/api/users/create",
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
async fn admins_can_read_update_and_change_user_status(pool: PgPool) {
    let admin_id = insert_user(
        &pool,
        "admin@example.com",
        "admin-password",
        "admin",
        "active",
    )
    .await;
    let student_id = insert_user(
        &pool,
        "student@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let app = app(pool);
    let admin_cookie = login_cookie(&app, "admin@example.com", "admin-password").await;

    let fetched = request(
        &app,
        Method::GET,
        &format!("/api/users/{student_id}"),
        None,
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(fetched.status(), StatusCode::OK);
    assert_eq!(response_json(fetched).await["email"], "student@example.com");

    let missing = request(
        &app,
        Method::GET,
        &format!("/api/users/{}", Uuid::now_v7()),
        None,
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);

    let updated = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{student_id}"),
        Some(json!({ "name": "Renamed Student", "email": "Renamed@Example.com" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    let updated = response_json(updated).await;
    assert_eq!(updated["name"], "Renamed Student");
    assert_eq!(updated["email"], "renamed@example.com");

    let deactivated = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{student_id}/status"),
        Some(json!({ "status": "inactive" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(deactivated.status(), StatusCode::OK);
    assert_eq!(response_json(deactivated).await["status"], "inactive");

    let reactivated = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{student_id}/status"),
        Some(json!({ "status": "active" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(reactivated.status(), StatusCode::OK);
    assert_eq!(response_json(reactivated).await["status"], "active");

    let invalid_status = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{student_id}/status"),
        Some(json!({ "status": "deleted" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(invalid_status.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let self_status = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{admin_id}/status"),
        Some(json!({ "status": "inactive" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(self_status.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn admins_cannot_modify_super_admins_and_non_admins_are_forbidden(pool: PgPool) {
    let super_admin_id = insert_user(
        &pool,
        "root@example.com",
        "root-password",
        "super_admin",
        "active",
    )
    .await;
    insert_user(
        &pool,
        "admin@example.com",
        "admin-password",
        "admin",
        "active",
    )
    .await;
    let student_id = insert_user(
        &pool,
        "student@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let app = app(pool);
    let admin_cookie = login_cookie(&app, "admin@example.com", "admin-password").await;
    let student_cookie = login_cookie(&app, "student@example.com", "student-password").await;

    let deactivate_root = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{super_admin_id}/status"),
        Some(json!({ "status": "inactive" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(deactivate_root.status(), StatusCode::FORBIDDEN);

    let rename_root = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{super_admin_id}"),
        Some(json!({ "name": "Hijacked" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(rename_root.status(), StatusCode::FORBIDDEN);

    let promote_student = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{student_id}"),
        Some(json!({ "role": "SuperAdmin" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(promote_student.status(), StatusCode::FORBIDDEN);

    for (method, uri, body) in [
        (Method::GET, "/api/users".to_string(), None),
        (Method::GET, format!("/api/users/{student_id}"), None),
        (
            Method::PATCH,
            format!("/api/users/{student_id}"),
            Some(json!({ "name": "Self Rename" })),
        ),
        (
            Method::PATCH,
            format!("/api/users/{super_admin_id}/status"),
            Some(json!({ "status": "inactive" })),
        ),
    ] {
        let response = request(&app, method, &uri, body, Some(&student_cookie)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn deactivated_users_lose_their_session_and_role_changes_apply_immediately(pool: PgPool) {
    insert_user(
        &pool,
        "admin@example.com",
        "admin-password",
        "admin",
        "active",
    )
    .await;
    let student_id = insert_user(
        &pool,
        "student@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let app = app(pool);
    let admin_cookie = login_cookie(&app, "admin@example.com", "admin-password").await;
    let student_cookie = login_cookie(&app, "student@example.com", "student-password").await;

    let before = request(
        &app,
        Method::GET,
        "/api/auth/me",
        None,
        Some(&student_cookie),
    )
    .await;
    assert_eq!(before.status(), StatusCode::OK);

    let promoted = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{student_id}"),
        Some(json!({ "role": "Admin" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(promoted.status(), StatusCode::OK);

    let list_as_promoted =
        request(&app, Method::GET, "/api/users", None, Some(&student_cookie)).await;
    assert_eq!(list_as_promoted.status(), StatusCode::OK);

    let deactivated = request(
        &app,
        Method::PATCH,
        &format!("/api/users/{student_id}/status"),
        Some(json!({ "status": "inactive" })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(deactivated.status(), StatusCode::OK);

    let after = request(
        &app,
        Method::GET,
        "/api/auth/me",
        None,
        Some(&student_cookie),
    )
    .await;
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response_json(after).await["code"], "INVALID_TOKEN");
}

#[sqlx::test(migrations = "./migrations")]
async fn login_ignores_email_case(pool: PgPool) {
    insert_user(
        &pool,
        "mixed@example.com",
        "correct-password",
        "student",
        "active",
    )
    .await;
    let app = app(pool);

    let response = request(
        &app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "email": "Mixed@Example.COM", "password": "correct-password" })),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
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
    let app = app(pool.clone());
    let cookie = login_cookie(&app, "user@example.com", "old-password").await;

    let changed = request(
        &app,
        Method::PATCH,
        "/api/users/change-password",
        Some(json!({ "new_password": "new-password" })),
        Some(&cookie),
    )
    .await;
    assert_eq!(changed.status(), StatusCode::NO_CONTENT);

    let old_login = request(
        &app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "email": "user@example.com", "password": "old-password" })),
        None,
    )
    .await;
    assert_eq!(old_login.status(), StatusCode::UNAUTHORIZED);

    let new_login = request(
        &app,
        Method::POST,
        "/api/auth/login",
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
    let student_id = insert_user(
        &pool,
        "student@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let app = app(pool.clone());
    let teacher_cookie = login_cookie(&app, "teacher@example.com", "teacher-password").await;
    let student_cookie = login_cookie(&app, "student@example.com", "student-password").await;

    let unauthenticated = request(
        &app,
        Method::GET,
        &format!("/api/courses/{}", Uuid::now_v7()),
        None,
        None,
    )
    .await;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let student_forbidden = request(
        &app,
        Method::GET,
        &format!("/api/courses/{}", Uuid::now_v7()),
        None,
        Some(&student_cookie),
    )
    .await;
    assert_eq!(student_forbidden.status(), StatusCode::FORBIDDEN);

    let created = request(
        &app,
        Method::POST,
        "/api/courses/create",
        Some(json!({
            "name": "Software Engineering",
            "code": "IS212",
            "teacher_id": teacher_id,
            "status": "active",
            "capacity": 25
        })),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created = response_json(created).await;
    let course_id = created["id"].as_str().expect("course id is present");

    let enrollment_payload = json!({ "student_id": student_id });
    let enrolled = request(
        &app,
        Method::POST,
        &format!("/api/courses/{course_id}/students"),
        Some(enrollment_payload.clone()),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(enrolled.status(), StatusCode::CREATED);

    let student_cannot_enroll = request(
        &app,
        Method::POST,
        &format!("/api/courses/{course_id}/students"),
        Some(enrollment_payload.clone()),
        Some(&student_cookie),
    )
    .await;
    assert_eq!(student_cannot_enroll.status(), StatusCode::FORBIDDEN);

    let duplicate = request(
        &app,
        Method::POST,
        &format!("/api/courses/{course_id}/students"),
        Some(enrollment_payload.clone()),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    for uri in [
        format!("/api/courses/{course_id}"),
        "/api/courses".to_string(),
    ] {
        let response = request(&app, Method::GET, &uri, None, Some(&teacher_cookie)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        if uri.ends_with(course_id) {
            assert_eq!(body["code"], "IS212");
        } else {
            assert_eq!(body["items"][0]["code"], "IS212");
        }
    }

    let missing = request(
        &app,
        Method::GET,
        &format!("/api/courses/{}", Uuid::now_v7()),
        None,
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}
