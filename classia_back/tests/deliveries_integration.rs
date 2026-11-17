//! HTTP tests for DEV-12 (submit/replace) and DEV-14 (grading), following the
//! minimum cases of the DEV-10 contract (#46).

use std::path::PathBuf;

use argon2::{Argon2, password_hash::PasswordHasher};
use axum::{
    Router,
    body::Body,
    http::{
        Method, Request, Response, StatusCode,
        header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
    },
};
use chrono::{Duration, Utc};
use classia_back::{JwtSettings, build_router};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "0123456789abcdef0123456789abcdef";
const PASSWORD: &str = "correct-password";
const BOUNDARY: &str = "classia-test-boundary";

/// The deliveries directory comes from `DELIVERIES_DIR`; every test process
/// shares one temporary directory.
fn files_dir() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("classia-it-deliveries-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&dir).expect("temp dir is created");
        // SAFETY: set once, before any router reads it, from a single init.
        unsafe { std::env::set_var("DELIVERIES_DIR", &dir) };
        dir
    })
    .clone()
}

fn app(pool: PgPool) -> Router {
    files_dir();
    build_router(
        pool,
        JwtSettings {
            secret: JWT_SECRET.to_string(),
            issuer: "classia".to_string(),
            expiration: 3600,
            mfa_encryption_key: None,
        },
    )
    .expect("test router builds")
}

async fn insert_user(pool: &PgPool, email: &str, role: &str) -> Uuid {
    let hash = Argon2::default()
        .hash_password(PASSWORD.as_bytes())
        .expect("password hashing succeeds")
        .to_string();
    sqlx::query_scalar(
        "INSERT INTO users (name, email, password, role, status, token)
         VALUES ('Test User', $1, $2, $3::user_role, 'active', '')
         RETURNING id",
    )
    .bind(email)
    .bind(hash)
    .bind(role)
    .fetch_one(pool)
    .await
    .expect("test user is inserted")
}

async fn insert_course(pool: &PgPool, teacher_id: Uuid, code: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO courses (name, code, teacher_id, status, capacity)
         VALUES ('Software Engineering', $2, $1, 'active', 20)
         RETURNING id",
    )
    .bind(teacher_id)
    .bind(code)
    .fetch_one(pool)
    .await
    .expect("test course is inserted")
}

async fn enroll(pool: &PgPool, course_id: Uuid, student_id: Uuid) {
    sqlx::query("INSERT INTO course_enrollments (course_id, student_id) VALUES ($1, $2)")
        .bind(course_id)
        .bind(student_id)
        .execute(pool)
        .await
        .expect("student is enrolled");
}

async fn insert_activity(pool: &PgPool, course_id: Uuid, status: &str, due_in: Duration) -> Uuid {
    let published_at = (status == "published").then(Utc::now);
    sqlx::query_scalar(
        "INSERT INTO activities (course_id, title, description, due_at, status, published_at)
         VALUES ($1, 'Workshop', 'Description', $2, $3::activity_status, $4)
         RETURNING id",
    )
    .bind(course_id)
    .bind(Utc::now() + due_in)
    .bind(status)
    .bind(published_at)
    .fetch_one(pool)
    .await
    .expect("test activity is inserted")
}

async fn expire_activity(pool: &PgPool, activity_id: Uuid) {
    sqlx::query("UPDATE activities SET due_at = now() - interval '1 minute' WHERE id = $1")
        .bind(activity_id)
        .execute(pool)
        .await
        .expect("activity expires");
}

fn docx(content: &str) -> Vec<u8> {
    let mut bytes = b"PK\x03\x04".to_vec();
    bytes.extend_from_slice(content.as_bytes());
    bytes
}

fn multipart(file: Option<(&str, &[u8])>, expected_version: Option<&str>) -> Vec<u8> {
    let mut body = Vec::new();
    if let Some((name, bytes)) = file {
        body.extend_from_slice(
            format!(
                "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    if let Some(version) = expected_version {
        body.extend_from_slice(
            format!(
                "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"expected_version\"\r\n\r\n{version}\r\n"
            )
            .as_bytes(),
        );
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    body
}

async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    cookie: &str,
    content_type: Option<String>,
    body: Vec<u8>,
) -> Response<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(COOKIE, cookie);
    if let Some(content_type) = content_type {
        builder = builder.header(CONTENT_TYPE, content_type);
    }
    app.clone()
        .oneshot(builder.body(Body::from(body)).expect("request builds"))
        .await
        .expect("router handles request")
}

async fn upload(
    app: &Router,
    method: Method,
    uri: &str,
    cookie: &str,
    body: Vec<u8>,
) -> Response<Body> {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    send(app, method, uri, cookie, Some(content_type), body).await
}

async fn grade(app: &Router, delivery_id: &str, cookie: &str, body: Value) -> Response<Body> {
    send(
        app,
        Method::PUT,
        &format!("/api/deliveries/{delivery_id}/evaluation"),
        cookie,
        Some("application/json".into()),
        body.to_string().into_bytes(),
    )
    .await
}

async fn json_body(response: Response<Body>) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body is read")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("body is JSON")
}

async fn login(app: &Router, email: &str) -> String {
    let response = send(
        app,
        Method::POST,
        "/api/auth/login",
        "",
        Some("application/json".into()),
        json!({ "email": email, "password": PASSWORD })
            .to_string()
            .into_bytes(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    response
        .headers()
        .get(SET_COOKIE)
        .expect("login sets a cookie")
        .to_str()
        .expect("cookie is valid")
        .split(';')
        .next()
        .expect("cookie has a value")
        .to_string()
}

async fn audit_count(pool: &PgPool, action: &str, result: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM audit_events WHERE action = $1 AND result = $2::audit_result",
    )
    .bind(action)
    .bind(result)
    .fetch_one(pool)
    .await
    .expect("audit is counted")
}

async fn stored_file(pool: &PgPool, delivery_id: &str) -> String {
    sqlx::query_scalar("SELECT file_url FROM deliveries WHERE id = $1::uuid")
        .bind(delivery_id)
        .fetch_one(pool)
        .await
        .expect("delivery exists")
}

struct Fixture {
    app: Router,
    teacher: String,
    other_teacher: String,
    student: String,
    outsider: String,
    activity_id: Uuid,
    draft_id: Uuid,
}

async fn fixture(pool: &PgPool) -> Fixture {
    let teacher_id = insert_user(pool, "teacher@example.com", "teacher").await;
    let other_teacher_id = insert_user(pool, "other@example.com", "teacher").await;
    let student_id = insert_user(pool, "student@example.com", "student").await;
    insert_user(pool, "outsider@example.com", "student").await;
    let course_id = insert_course(pool, teacher_id, "IS212").await;
    insert_course(pool, other_teacher_id, "IS213").await;
    enroll(pool, course_id, student_id).await;
    let activity_id = insert_activity(pool, course_id, "published", Duration::days(2)).await;
    let draft_id = insert_activity(pool, course_id, "draft", Duration::days(2)).await;

    let app = app(pool.clone());
    Fixture {
        teacher: login(&app, "teacher@example.com").await,
        other_teacher: login(&app, "other@example.com").await,
        student: login(&app, "student@example.com").await,
        outsider: login(&app, "outsider@example.com").await,
        app,
        activity_id,
        draft_id,
    }
}

async fn submit(f: &Fixture, activity_id: Uuid, cookie: &str, body: Vec<u8>) -> Response<Body> {
    upload(
        &f.app,
        Method::POST,
        &format!("/api/activities/{activity_id}/deliveries"),
        cookie,
        body,
    )
    .await
}

#[sqlx::test(migrations = "./migrations")]
async fn submit_creates_delivery_and_enforces_contract_rules(pool: PgPool) {
    let f = fixture(&pool).await;
    let valid = multipart(Some(("Entrega.DOCX", &docx("v1"))), None);

    let forbidden = submit(&f, f.activity_id, &f.outsider, valid.clone()).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let teacher = submit(&f, f.activity_id, &f.teacher, valid.clone()).await;
    assert_eq!(teacher.status(), StatusCode::FORBIDDEN);
    let draft = submit(&f, f.draft_id, &f.student, valid.clone()).await;
    assert_eq!(json_body(draft).await["code"], "ACTIVITY_NOT_FOUND");

    let not_docx = submit(
        &f,
        f.activity_id,
        &f.student,
        multipart(Some(("notes.pdf", &docx("pdf"))), None),
    )
    .await;
    assert_eq!(json_body(not_docx).await["code"], "INVALID_FILE_TYPE");
    let fake_docx = submit(
        &f,
        f.activity_id,
        &f.student,
        multipart(Some(("fake.docx", b"plain text")), None),
    )
    .await;
    assert_eq!(json_body(fake_docx).await["code"], "INVALID_FILE_TYPE");
    let empty = submit(&f, f.activity_id, &f.student, multipart(None, None)).await;
    assert_eq!(json_body(empty).await["code"], "VALIDATION_ERROR");

    let mut large = docx("");
    large.resize(20 * 1024 * 1024 + 1, b'x');
    let too_large = submit(
        &f,
        f.activity_id,
        &f.student,
        multipart(Some(("big.docx", &large)), None),
    )
    .await;
    assert_eq!(too_large.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(too_large).await["code"], "FILE_TOO_LARGE");

    let created = submit(&f, f.activity_id, &f.student, valid.clone()).await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let delivery = json_body(created).await;
    assert_eq!(delivery["status"], "submitted");
    assert_eq!(delivery["version"], 1);
    assert_eq!(delivery["evaluation"], Value::Null);
    assert_eq!(delivery["student"]["email"], "student@example.com");
    assert!(delivery.get("file_url").is_none());
    let file = stored_file(&pool, delivery["id"].as_str().unwrap()).await;
    assert_eq!(std::fs::read(files_dir().join(&file)).unwrap(), docx("v1"));

    let duplicate = submit(&f, f.activity_id, &f.student, valid).await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
    assert_eq!(
        json_body(duplicate).await["code"],
        "DELIVERY_ALREADY_EXISTS"
    );

    assert_eq!(audit_count(&pool, "submit_delivery", "success").await, 1);
    assert_eq!(audit_count(&pool, "submit_delivery", "denied").await, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn submit_after_due_date_is_rejected_without_leaving_a_file(pool: PgPool) {
    let f = fixture(&pool).await;
    expire_activity(&pool, f.activity_id).await;
    let before = std::fs::read_dir(files_dir()).unwrap().count();

    let late = submit(
        &f,
        f.activity_id,
        &f.student,
        multipart(Some(("late.docx", &docx("late"))), None),
    )
    .await;
    assert_eq!(late.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(late).await["code"], "DUE_DATE_PASSED");
    assert_eq!(std::fs::read_dir(files_dir()).unwrap().count(), before);
}

#[sqlx::test(migrations = "./migrations")]
async fn replace_increments_version_and_clears_a_previous_grade(pool: PgPool) {
    let f = fixture(&pool).await;
    let created = json_body(
        submit(
            &f,
            f.activity_id,
            &f.student,
            multipart(Some(("a.docx", &docx("v1"))), None),
        )
        .await,
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();
    let first_file = stored_file(&pool, &id).await;

    let graded = grade(
        &f.app,
        &id,
        &f.teacher,
        json!({ "score": 3.5, "feedback": "Bien", "expected_version": 1 }),
    )
    .await;
    assert_eq!(graded.status(), StatusCode::OK);

    let uri = format!("/api/deliveries/{id}");
    let stale = upload(
        &f.app,
        Method::PUT,
        &uri,
        &f.student,
        multipart(Some(("b.docx", &docx("v2"))), Some("1")),
    )
    .await;
    assert_eq!(json_body(stale).await["code"], "DELIVERY_VERSION_CONFLICT");

    let missing_version = upload(
        &f.app,
        Method::PUT,
        &uri,
        &f.student,
        multipart(Some(("b.docx", &docx("v2"))), None),
    )
    .await;
    assert_eq!(json_body(missing_version).await["code"], "VALIDATION_ERROR");

    let not_author = upload(
        &f.app,
        Method::PUT,
        &uri,
        &f.outsider,
        multipart(Some(("b.docx", &docx("v2"))), Some("2")),
    )
    .await;
    assert_eq!(json_body(not_author).await["code"], "DELIVERY_NOT_FOUND");

    let replaced = upload(
        &f.app,
        Method::PUT,
        &uri,
        &f.student,
        multipart(Some(("b.docx", &docx("v2"))), Some("2")),
    )
    .await;
    assert_eq!(replaced.status(), StatusCode::OK);
    let replaced = json_body(replaced).await;
    assert_eq!(replaced["status"], "replaced");
    assert_eq!(replaced["version"], 3);
    assert_eq!(replaced["evaluation"], Value::Null);

    let new_file = stored_file(&pool, &id).await;
    assert_ne!(new_file, first_file);
    assert!(!files_dir().join(&first_file).exists());
    assert_eq!(
        std::fs::read(files_dir().join(&new_file)).unwrap(),
        docx("v2")
    );

    let previous: Value = sqlx::query_scalar(
        "SELECT details->'previous_evaluation' FROM audit_events
         WHERE action = 'replace_delivery' AND result = 'success'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(previous, json!({ "score": 3.5, "feedback": "Bien" }));
    assert_eq!(audit_count(&pool, "replace_delivery", "denied").await, 1);

    expire_activity(&pool, f.activity_id).await;
    let late = upload(
        &f.app,
        Method::PUT,
        &uri,
        &f.student,
        multipart(Some(("c.docx", &docx("v3"))), Some("3")),
    )
    .await;
    assert_eq!(json_body(late).await["code"], "DUE_DATE_PASSED");
    assert_eq!(stored_file(&pool, &id).await, new_file);
}

#[sqlx::test(migrations = "./migrations")]
async fn grading_creates_corrects_and_audits_the_evaluation(pool: PgPool) {
    let f = fixture(&pool).await;
    let created = json_body(
        submit(
            &f,
            f.activity_id,
            &f.student,
            multipart(Some(("a.docx", &docx("v1"))), None),
        )
        .await,
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();

    for score in [json!(5.1), json!(-0.1), json!(4.25)] {
        let invalid = grade(
            &f.app,
            &id,
            &f.teacher,
            json!({ "score": score, "expected_version": 1 }),
        )
        .await;
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
        assert_eq!(json_body(invalid).await["code"], "VALIDATION_ERROR");
    }
    let malformed = grade(&f.app, &id, &f.teacher, json!({ "score": "4" })).await;
    assert_eq!(json_body(malformed).await["code"], "VALIDATION_ERROR");

    let other = grade(
        &f.app,
        &id,
        &f.other_teacher,
        json!({ "score": 4.0, "expected_version": 1 }),
    )
    .await;
    assert_eq!(other.status(), StatusCode::FORBIDDEN);
    let student = grade(
        &f.app,
        &id,
        &f.student,
        json!({ "score": 5.0, "expected_version": 1 }),
    )
    .await;
    assert_eq!(student.status(), StatusCode::FORBIDDEN);
    let unknown = grade(
        &f.app,
        &Uuid::now_v7().to_string(),
        &f.teacher,
        json!({ "score": 4.0, "expected_version": 1 }),
    )
    .await;
    assert_eq!(json_body(unknown).await["code"], "DELIVERY_NOT_FOUND");

    let graded = grade(
        &f.app,
        &id,
        &f.teacher,
        json!({ "score": 4.5, "feedback": "  Buen modelo.  ", "expected_version": 1 }),
    )
    .await;
    assert_eq!(graded.status(), StatusCode::OK);
    let graded = json_body(graded).await;
    assert_eq!(graded["status"], "graded");
    assert_eq!(graded["version"], 2);
    assert_eq!(graded["evaluation"]["score"], 4.5);
    assert_eq!(graded["evaluation"]["feedback"], "Buen modelo.");

    let stale = grade(
        &f.app,
        &id,
        &f.teacher,
        json!({ "score": 1.0, "expected_version": 1 }),
    )
    .await;
    assert_eq!(stale.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(stale).await["code"], "DELIVERY_VERSION_CONFLICT");

    let corrected = grade(
        &f.app,
        &id,
        &f.teacher,
        json!({ "score": 3.0, "feedback": "   ", "expected_version": 2 }),
    )
    .await;
    let corrected = json_body(corrected).await;
    assert_eq!(corrected["version"], 3);
    assert_eq!(corrected["evaluation"]["score"], 3.0);
    assert_eq!(corrected["evaluation"]["feedback"], Value::Null);
    assert_eq!(
        corrected["evaluation"]["graded_at"],
        graded["evaluation"]["graded_at"]
    );

    let own = json_body(
        send(
            &f.app,
            Method::GET,
            &format!("/api/activities/{}/deliveries/me", f.activity_id),
            &f.student,
            None,
            Vec::new(),
        )
        .await,
    )
    .await;
    assert_eq!(own["delivery"]["evaluation"]["score"], 3.0);

    let correction: Value = sqlx::query_scalar(
        "SELECT details FROM audit_events
         WHERE action = 'grade_delivery' AND result = 'success'
         ORDER BY occurred_at DESC, id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(correction["previous"]["score"], 4.5);
    assert_eq!(correction["new"]["score"], 3.0);
    assert_eq!(audit_count(&pool, "grade_delivery", "success").await, 2);
    assert_eq!(audit_count(&pool, "grade_delivery", "denied").await, 3);
}
