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

async fn create_course(
    app: &Router,
    cookie: &str,
    code: &str,
    teacher_id: Uuid,
    capacity: i16,
) -> Value {
    let response = request(
        app,
        Method::POST,
        "/api/courses",
        Some(json!({
            "name": "Software Engineering",
            "code": code,
            "teacher_id": teacher_id,
            "status": "active",
            "capacity": capacity
        })),
        Some(cookie),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await
}

async fn enroll(app: &Router, cookie: &str, course_id: &str, student_id: Uuid) -> Response<Body> {
    request(
        app,
        Method::POST,
        &format!("/api/courses/{course_id}/students"),
        Some(json!({ "student_id": student_id })),
        Some(cookie),
    )
    .await
}

#[sqlx::test(migrations = "./migrations")]
async fn courses_are_created_listed_and_read_by_role(pool: PgPool) {
    let teacher_id = insert_user(
        &pool,
        "teacher@example.com",
        "teacher-password",
        "teacher",
        "active",
    )
    .await;
    let other_teacher_id = insert_user(
        &pool,
        "other@example.com",
        "other-password",
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
    insert_user(
        &pool,
        "admin@example.com",
        "admin-password",
        "admin",
        "active",
    )
    .await;
    let app = app(pool.clone());
    let teacher_cookie = login_cookie(&app, "teacher@example.com", "teacher-password").await;
    let other_cookie = login_cookie(&app, "other@example.com", "other-password").await;
    let student_cookie = login_cookie(&app, "student@example.com", "student-password").await;
    let admin_cookie = login_cookie(&app, "admin@example.com", "admin-password").await;

    let unauthenticated = request(&app, Method::GET, "/api/courses", None, None).await;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let student_create = request(
        &app,
        Method::POST,
        "/api/courses",
        Some(json!({
            "name": "Course",
            "code": "ST1",
            "teacher_id": teacher_id,
            "status": "active",
            "capacity": 5
        })),
        Some(&student_cookie),
    )
    .await;
    assert_eq!(student_create.status(), StatusCode::FORBIDDEN);

    let teacher_for_other = request(
        &app,
        Method::POST,
        "/api/courses",
        Some(json!({
            "name": "Course",
            "code": "OT1",
            "teacher_id": other_teacher_id,
            "status": "active",
            "capacity": 5
        })),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(teacher_for_other.status(), StatusCode::FORBIDDEN);

    let created = create_course(&app, &teacher_cookie, " is212 ", teacher_id, 25).await;
    let course_id = created["id"]
        .as_str()
        .expect("course id is present")
        .to_string();
    assert_eq!(created["code"], "IS212");
    assert_eq!(created["teacher"]["id"], teacher_id.to_string());
    assert_eq!(created["enrolled_count"], 0);
    assert!(created.get("password").is_none());

    let duplicate = request(
        &app,
        Method::POST,
        "/api/courses",
        Some(json!({
            "name": "Copy",
            "code": "IS212",
            "teacher_id": teacher_id,
            "status": "active",
            "capacity": 5
        })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
    assert_eq!(response_json(duplicate).await["code"], "COURSE_CODE_EXISTS");

    let student_as_teacher = request(
        &app,
        Method::POST,
        "/api/courses",
        Some(json!({
            "name": "Course",
            "code": "BAD1",
            "teacher_id": student_id,
            "status": "active",
            "capacity": 5
        })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(student_as_teacher.status(), StatusCode::BAD_REQUEST);

    let zero_capacity = request(
        &app,
        Method::POST,
        "/api/courses",
        Some(json!({
            "name": "Course",
            "code": "ZERO",
            "teacher_id": teacher_id,
            "status": "active",
            "capacity": 0
        })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(zero_capacity.status(), StatusCode::BAD_REQUEST);

    for (cookie, expected) in [
        (&admin_cookie, 1),
        (&teacher_cookie, 1),
        (&other_cookie, 0),
        (&student_cookie, 0),
    ] {
        let response = request(&app, Method::GET, "/api/courses", None, Some(cookie)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(
            body["items"].as_array().expect("items is a list").len(),
            expected
        );
    }

    let course_uri = format!("/api/courses/{course_id}");
    let before_enrollment =
        request(&app, Method::GET, &course_uri, None, Some(&student_cookie)).await;
    assert_eq!(before_enrollment.status(), StatusCode::FORBIDDEN);
    let other_teacher = request(&app, Method::GET, &course_uri, None, Some(&other_cookie)).await;
    assert_eq!(other_teacher.status(), StatusCode::FORBIDDEN);

    assert_eq!(
        enroll(&app, &teacher_cookie, &course_id, student_id)
            .await
            .status(),
        StatusCode::CREATED
    );

    let enrolled_view = request(&app, Method::GET, &course_uri, None, Some(&student_cookie)).await;
    assert_eq!(enrolled_view.status(), StatusCode::OK);
    assert_eq!(response_json(enrolled_view).await["enrolled_count"], 1);

    let student_courses = request(
        &app,
        Method::GET,
        "/api/courses",
        None,
        Some(&student_cookie),
    )
    .await;
    let student_courses = response_json(student_courses).await;
    assert_eq!(student_courses["items"][0]["code"], "IS212");
    assert_eq!(student_courses["items"][0]["enrolled_count"], 1);

    let missing = request(
        &app,
        Method::GET,
        &format!("/api/courses/{}", Uuid::now_v7()),
        None,
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(response_json(missing).await["code"], "COURSE_NOT_FOUND");
}

#[sqlx::test(migrations = "./migrations")]
async fn course_updates_enforce_ownership_teacher_changes_and_capacity(pool: PgPool) {
    let teacher_id = insert_user(
        &pool,
        "teacher@example.com",
        "teacher-password",
        "teacher",
        "active",
    )
    .await;
    let new_teacher_id = insert_user(
        &pool,
        "new@example.com",
        "new-password",
        "teacher",
        "active",
    )
    .await;
    insert_user(
        &pool,
        "other@example.com",
        "other-password",
        "teacher",
        "active",
    )
    .await;
    let first_student = insert_user(
        &pool,
        "s1@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let second_student = insert_user(
        &pool,
        "s2@example.com",
        "student-password",
        "student",
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
    let app = app(pool.clone());
    let teacher_cookie = login_cookie(&app, "teacher@example.com", "teacher-password").await;
    let other_cookie = login_cookie(&app, "other@example.com", "other-password").await;
    let admin_cookie = login_cookie(&app, "admin@example.com", "admin-password").await;

    let course = create_course(&app, &admin_cookie, "UPD1", teacher_id, 5).await;
    let course_id = course["id"]
        .as_str()
        .expect("course id is present")
        .to_string();
    let course_uri = format!("/api/courses/{course_id}");
    create_course(&app, &admin_cookie, "UPD2", teacher_id, 5).await;

    let renamed = request(
        &app,
        Method::PATCH,
        &course_uri,
        Some(json!({ "name": "Renamed", "status": "inactive" })),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(renamed.status(), StatusCode::OK);
    let renamed = response_json(renamed).await;
    assert_eq!(renamed["name"], "Renamed");
    assert_eq!(renamed["status"], "inactive");
    assert_eq!(renamed["code"], "UPD1");

    let not_owner = request(
        &app,
        Method::PATCH,
        &course_uri,
        Some(json!({ "name": "Stolen" })),
        Some(&other_cookie),
    )
    .await;
    assert_eq!(not_owner.status(), StatusCode::FORBIDDEN);

    let teacher_reassigns = request(
        &app,
        Method::PATCH,
        &course_uri,
        Some(json!({ "teacher_id": new_teacher_id })),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(teacher_reassigns.status(), StatusCode::FORBIDDEN);

    let duplicate_code = request(
        &app,
        Method::PATCH,
        &course_uri,
        Some(json!({ "code": "UPD2" })),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(duplicate_code.status(), StatusCode::CONFLICT);

    request(
        &app,
        Method::PATCH,
        &course_uri,
        Some(json!({ "status": "active" })),
        Some(&teacher_cookie),
    )
    .await;
    for student in [first_student, second_student] {
        assert_eq!(
            enroll(&app, &teacher_cookie, &course_id, student)
                .await
                .status(),
            StatusCode::CREATED
        );
    }

    let below_enrollment = request(
        &app,
        Method::PATCH,
        &course_uri,
        Some(json!({ "capacity": 1 })),
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(below_enrollment.status(), StatusCode::CONFLICT);
    assert_eq!(
        response_json(below_enrollment).await["code"],
        "CAPACITY_BELOW_ENROLLMENT"
    );

    let reassigned = request(
        &app,
        Method::PATCH,
        &course_uri,
        Some(json!({ "teacher_id": new_teacher_id, "capacity": 2 })),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(reassigned.status(), StatusCode::OK);
    let reassigned = response_json(reassigned).await;
    assert_eq!(reassigned["teacher"]["id"], new_teacher_id.to_string());
    assert_eq!(reassigned["capacity"], 2);

    let former_owner = request(&app, Method::GET, &course_uri, None, Some(&teacher_cookie)).await;
    assert_eq!(former_owner.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn teachers_enroll_students_within_the_contract_rules(pool: PgPool) {
    let teacher_id = insert_user(
        &pool,
        "teacher@example.com",
        "teacher-password",
        "teacher",
        "active",
    )
    .await;
    insert_user(
        &pool,
        "other@example.com",
        "other-password",
        "teacher",
        "active",
    )
    .await;
    let first_student = insert_user(
        &pool,
        "ana@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let second_student = insert_user(
        &pool,
        "bruno@example.com",
        "student-password",
        "student",
        "active",
    )
    .await;
    let inactive_student = insert_user(
        &pool,
        "inactive@example.com",
        "student-password",
        "student",
        "inactive",
    )
    .await;
    let admin_id = insert_user(
        &pool,
        "admin@example.com",
        "admin-password",
        "admin",
        "active",
    )
    .await;
    let app = app(pool.clone());
    let teacher_cookie = login_cookie(&app, "teacher@example.com", "teacher-password").await;
    let other_cookie = login_cookie(&app, "other@example.com", "other-password").await;
    let student_cookie = login_cookie(&app, "ana@example.com", "student-password").await;
    let admin_cookie = login_cookie(&app, "admin@example.com", "admin-password").await;

    let course = create_course(&app, &teacher_cookie, "ENR1", teacher_id, 1).await;
    let course_id = course["id"]
        .as_str()
        .expect("course id is present")
        .to_string();
    let students_uri = format!("/api/courses/{course_id}/students");
    let available_uri = format!("/api/courses/{course_id}/available-students");

    let available = request(
        &app,
        Method::GET,
        &available_uri,
        None,
        Some(&teacher_cookie),
    )
    .await;
    assert_eq!(available.status(), StatusCode::OK);
    let available = response_json(available).await;
    let emails: Vec<&str> = available["items"]
        .as_array()
        .expect("items is a list")
        .iter()
        .map(|user| user["email"].as_str().expect("email is present"))
        .collect();
    assert_eq!(emails, ["ana@example.com", "bruno@example.com"]);
    assert!(available["items"][0].get("password").is_none());

    let searched = request(
        &app,
        Method::GET,
        &format!("{available_uri}?q=BRUNO"),
        None,
        Some(&teacher_cookie),
    )
    .await;
    let searched = response_json(searched).await;
    assert_eq!(
        searched["items"].as_array().expect("items is a list").len(),
        1
    );
    assert_eq!(searched["items"][0]["email"], "bruno@example.com");

    for (cookie, uri) in [
        (&admin_cookie, &available_uri),
        (&other_cookie, &available_uri),
        (&student_cookie, &available_uri),
    ] {
        let response = request(&app, Method::GET, uri, None, Some(cookie)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
    }

    for (cookie, student) in [
        (&admin_cookie, first_student),
        (&other_cookie, first_student),
        (&student_cookie, first_student),
    ] {
        assert_eq!(
            enroll(&app, cookie, &course_id, student).await.status(),
            StatusCode::FORBIDDEN
        );
    }

    for (student, status, code) in [
        (Uuid::now_v7(), StatusCode::NOT_FOUND, "STUDENT_NOT_FOUND"),
        (admin_id, StatusCode::BAD_REQUEST, "INVALID_STUDENT_ROLE"),
        (inactive_student, StatusCode::CONFLICT, "STUDENT_INACTIVE"),
    ] {
        let response = enroll(&app, &teacher_cookie, &course_id, student).await;
        assert_eq!(response.status(), status, "{code}");
        assert_eq!(response_json(response).await["code"], code);
    }

    let enrolled = enroll(&app, &teacher_cookie, &course_id, first_student).await;
    assert_eq!(enrolled.status(), StatusCode::CREATED);
    let enrolled = response_json(enrolled).await;
    assert_eq!(enrolled["course_id"], course_id);
    assert_eq!(enrolled["student"]["id"], first_student.to_string());
    assert_eq!(enrolled["student"]["role"], "Student");
    assert_eq!(enrolled["student"]["status"], "active");
    assert!(enrolled["enrolled_at"].is_string());

    let duplicate = enroll(&app, &teacher_cookie, &course_id, first_student).await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
    assert_eq!(
        response_json(duplicate).await["code"],
        "STUDENT_ALREADY_ENROLLED"
    );

    let full = enroll(&app, &teacher_cookie, &course_id, second_student).await;
    assert_eq!(full.status(), StatusCode::CONFLICT);
    assert_eq!(response_json(full).await["code"], "COURSE_CAPACITY_REACHED");

    for cookie in [&teacher_cookie, &admin_cookie] {
        let response = request(&app, Method::GET, &students_uri, None, Some(cookie)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["items"].as_array().expect("items is a list").len(), 1);
        assert_eq!(body["items"][0]["email"], "ana@example.com");
        assert_eq!(body["items"][0]["role"], "Student");
        assert!(body["items"][0].get("password").is_none());
    }
    for cookie in [&other_cookie, &student_cookie] {
        let response = request(&app, Method::GET, &students_uri, None, Some(cookie)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    request(
        &app,
        Method::PATCH,
        &format!("/api/courses/{course_id}"),
        Some(json!({ "status": "inactive", "capacity": 5 })),
        Some(&teacher_cookie),
    )
    .await;
    let inactive_course = enroll(&app, &teacher_cookie, &course_id, second_student).await;
    assert_eq!(inactive_course.status(), StatusCode::CONFLICT);
    assert_eq!(
        response_json(inactive_course).await["code"],
        "COURSE_NOT_ACTIVE"
    );

    let missing = enroll(
        &app,
        &teacher_cookie,
        &Uuid::now_v7().to_string(),
        second_student,
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_enrollments_never_exceed_capacity(pool: PgPool) {
    let teacher_id = insert_user(
        &pool,
        "teacher@example.com",
        "teacher-password",
        "teacher",
        "active",
    )
    .await;
    let mut students = Vec::new();
    for index in 0..5 {
        students.push(
            insert_user(
                &pool,
                &format!("student{index}@example.com"),
                "student-password",
                "student",
                "active",
            )
            .await,
        );
    }
    let app = app(pool.clone());
    let teacher_cookie = login_cookie(&app, "teacher@example.com", "teacher-password").await;
    let course = create_course(&app, &teacher_cookie, "RACE", teacher_id, 2).await;
    let course_id = course["id"]
        .as_str()
        .expect("course id is present")
        .to_string();

    let attempts = students.into_iter().map(|student| {
        let app = app.clone();
        let cookie = teacher_cookie.clone();
        let course_id = course_id.clone();
        tokio::spawn(async move { enroll(&app, &cookie, &course_id, student).await.status() })
    });
    let mut created = 0;
    for attempt in attempts.collect::<Vec<_>>() {
        let status = attempt.await.expect("enrollment task completes");
        if status == StatusCode::CREATED {
            created += 1;
        } else {
            assert_eq!(status, StatusCode::CONFLICT);
        }
    }
    assert_eq!(created, 2);

    let enrolled: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM course_enrollments")
        .fetch_one(&pool)
        .await
        .expect("enrollments are counted");
    assert_eq!(enrolled, 2);
}
