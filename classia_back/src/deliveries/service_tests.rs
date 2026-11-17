use std::{fs, path::PathBuf, sync::Arc};

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{build_service, error::DeliveryError, models::DeliveryStatus};
use crate::{
    activities::{self, ActivityError},
    courses,
    error::AppError,
    users::{self, UserRole},
};

async fn insert_user(pool: &PgPool, email: &str, role: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO users (name, email, password, role, status, token)
         VALUES ('Test User', $1, 'hash', $2::user_role, 'active', '')
         RETURNING id",
    )
    .bind(email)
    .bind(role)
    .fetch_one(pool)
    .await
    .expect("test user is inserted")
}

async fn insert_course(pool: &PgPool, teacher_id: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO courses (name, code, teacher_id, status, capacity)
         VALUES ('Software Engineering', 'IS212', $1, 'active', 20)
         RETURNING id",
    )
    .bind(teacher_id)
    .fetch_one(pool)
    .await
    .expect("test course is inserted")
}

async fn enroll_student(pool: &PgPool, course_id: Uuid, student_id: Uuid) {
    sqlx::query("INSERT INTO course_enrollments (course_id, student_id) VALUES ($1, $2)")
        .bind(course_id)
        .bind(student_id)
        .execute(pool)
        .await
        .expect("test student is enrolled");
}

async fn insert_activity(pool: &PgPool, course_id: Uuid, status: &str) -> Uuid {
    let published_at = (status == "published").then(Utc::now);

    sqlx::query_scalar(
        "INSERT INTO activities
            (course_id, title, description, due_at, status, published_at)
         VALUES ($1, 'Workshop', 'Test description', $2, $3::activity_status, $4)
         RETURNING id",
    )
    .bind(course_id)
    .bind(Utc::now() + Duration::days(2))
    .bind(status)
    .bind(published_at)
    .fetch_one(pool)
    .await
    .expect("test activity is inserted")
}

async fn insert_delivery(
    pool: &PgPool,
    activity_id: Uuid,
    student_id: Uuid,
    file_url: &str,
    status: &str,
    updated_at: DateTime<Utc>,
) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO deliveries (activity_id, student_id, file_url, status, updated_at)
         VALUES ($1, $2, $3, $4::delivery_status, $5)
         RETURNING id",
    )
    .bind(activity_id)
    .bind(student_id)
    .bind(file_url)
    .bind(status)
    .bind(updated_at)
    .fetch_one(pool)
    .await
    .expect("test delivery is inserted")
}

async fn insert_evaluation(
    pool: &PgPool,
    delivery_id: Uuid,
    teacher_id: Uuid,
    score: f64,
    feedback: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO evaluations (delivery_id, score, feedback, teacher_id)
         VALUES ($1, $2::FLOAT8::NUMERIC(3, 1), $3, $4)",
    )
    .bind(delivery_id)
    .bind(score)
    .bind(feedback)
    .bind(teacher_id)
    .execute(pool)
    .await
    .expect("test evaluation is inserted");
}

fn temp_files_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("classia-deliveries-{}", Uuid::now_v7()));
    fs::create_dir_all(&dir).expect("temp dir is created");
    dir
}

fn service(pool: &PgPool, files_dir: PathBuf) -> super::DeliveryService {
    let user_service = Arc::new(users::build_service(pool.clone()));
    let course_service = Arc::new(courses::build_service(pool.clone(), user_service));
    let activity_service = Arc::new(activities::build_service(pool.clone(), course_service));

    build_service(pool.clone(), activity_service, files_dir)
}

#[test]
fn delivery_errors_map_to_contract_codes() {
    let cases = [
        (
            DeliveryError::DeliveryNotFound,
            StatusCode::NOT_FOUND,
            "DELIVERY_NOT_FOUND",
        ),
        (
            DeliveryError::FileUnavailable,
            StatusCode::INTERNAL_SERVER_ERROR,
            "FILE_UNAVAILABLE",
        ),
        (DeliveryError::Forbidden, StatusCode::FORBIDDEN, "FORBIDDEN"),
        (
            DeliveryError::Activity(ActivityError::ActivityNotFound),
            StatusCode::NOT_FOUND,
            "ACTIVITY_NOT_FOUND",
        ),
    ];

    for (error, status, code) in cases {
        let mapped: AppError = error.into();
        assert_eq!(mapped.status, status, "{code}");
        assert_eq!(mapped.code, code);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn student_sees_own_delivery_status(pool: PgPool) {
    let teacher_id = insert_user(&pool, "owner@example.com", "teacher").await;
    let graded_student = insert_user(&pool, "graded@example.com", "student").await;
    let pending_student = insert_user(&pool, "pending@example.com", "student").await;
    let new_student = insert_user(&pool, "new@example.com", "student").await;
    let course_id = insert_course(&pool, teacher_id).await;
    for student in [graded_student, pending_student, new_student] {
        enroll_student(&pool, course_id, student).await;
    }
    let activity_id = insert_activity(&pool, course_id, "published").await;

    let graded_id = insert_delivery(
        &pool,
        activity_id,
        graded_student,
        "a.docx",
        "graded",
        Utc::now(),
    )
    .await;
    insert_evaluation(&pool, graded_id, teacher_id, 4.5, Some("Buen modelo")).await;
    insert_delivery(
        &pool,
        activity_id,
        pending_student,
        "b.docx",
        "submitted",
        Utc::now(),
    )
    .await;

    let service = service(&pool, temp_files_dir());

    let graded = service
        .get_delivery_status(graded_student, &UserRole::Student, activity_id)
        .await
        .expect("the student can see their delivery")
        .expect("the delivery exists");
    assert_eq!(graded.status, DeliveryStatus::Graded);
    assert_eq!(graded.score, Some(4.5));
    assert_eq!(graded.feedback.as_deref(), Some("Buen modelo"));

    let pending = service
        .get_delivery_status(pending_student, &UserRole::Student, activity_id)
        .await
        .expect("the student can see their delivery")
        .expect("the delivery exists");
    assert_eq!(pending.status, DeliveryStatus::Submitted);
    assert_eq!(pending.score, None);

    assert!(
        service
            .get_delivery_status(new_student, &UserRole::Student, activity_id)
            .await
            .expect("a student without delivery is not an error")
            .is_none()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn delivery_status_rejects_other_roles_and_hidden_activities(pool: PgPool) {
    let teacher_id = insert_user(&pool, "owner@example.com", "teacher").await;
    let admin_id = insert_user(&pool, "admin@example.com", "admin").await;
    let student_id = insert_user(&pool, "student@example.com", "student").await;
    let outsider_id = insert_user(&pool, "outsider@example.com", "student").await;
    let course_id = insert_course(&pool, teacher_id).await;
    enroll_student(&pool, course_id, student_id).await;
    let published_id = insert_activity(&pool, course_id, "published").await;
    let draft_id = insert_activity(&pool, course_id, "draft").await;
    let service = service(&pool, temp_files_dir());

    for (user_id, role) in [(teacher_id, UserRole::Teacher), (admin_id, UserRole::Admin)] {
        assert!(matches!(
            service
                .get_delivery_status(user_id, &role, published_id)
                .await,
            Err(DeliveryError::Forbidden)
        ));
    }
    assert!(matches!(
        service
            .get_delivery_status(outsider_id, &UserRole::Student, published_id)
            .await,
        Err(DeliveryError::Activity(ActivityError::Forbidden))
    ));
    assert!(matches!(
        service
            .get_delivery_status(student_id, &UserRole::Student, draft_id)
            .await,
        Err(DeliveryError::Activity(ActivityError::ActivityNotFound))
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn owner_teacher_lists_deliveries_ordered_by_update(pool: PgPool) {
    let teacher_id = insert_user(&pool, "owner@example.com", "teacher").await;
    let other_teacher_id = insert_user(&pool, "other@example.com", "teacher").await;
    let admin_id = insert_user(&pool, "admin@example.com", "admin").await;
    let super_admin_id = insert_user(&pool, "super@example.com", "super_admin").await;
    let first_student = insert_user(&pool, "first@example.com", "student").await;
    let second_student = insert_user(&pool, "second@example.com", "student").await;
    let course_id = insert_course(&pool, teacher_id).await;
    enroll_student(&pool, course_id, first_student).await;
    enroll_student(&pool, course_id, second_student).await;
    let activity_id = insert_activity(&pool, course_id, "published").await;
    let empty_activity_id = insert_activity(&pool, course_id, "published").await;

    // The newer one is inserted first to prove the order comes from updated_at.
    let newer = insert_delivery(
        &pool,
        activity_id,
        second_student,
        "n.docx",
        "replaced",
        Utc::now() - Duration::hours(1),
    )
    .await;
    let older = insert_delivery(
        &pool,
        activity_id,
        first_student,
        "o.docx",
        "submitted",
        Utc::now() - Duration::hours(2),
    )
    .await;
    let service = service(&pool, temp_files_dir());

    let deliveries = service
        .list_activity_deliveries(teacher_id, &UserRole::Teacher, activity_id)
        .await
        .expect("the owner teacher can list deliveries");
    assert_eq!(
        deliveries.iter().map(|d| d.id).collect::<Vec<_>>(),
        vec![older, newer]
    );
    assert_eq!(deliveries[0].student_email, "first@example.com");

    assert!(
        service
            .list_activity_deliveries(teacher_id, &UserRole::Teacher, empty_activity_id)
            .await
            .expect("an activity without deliveries is not an error")
            .is_empty()
    );

    for (user_id, role) in [
        (other_teacher_id, UserRole::Teacher),
        (admin_id, UserRole::Admin),
        (super_admin_id, UserRole::SuperAdmin),
        (first_student, UserRole::Student),
    ] {
        assert!(
            service
                .list_activity_deliveries(user_id, &role, activity_id)
                .await
                .is_err(),
            "{user_id} must not list deliveries"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn owner_teacher_downloads_delivery_file(pool: PgPool) {
    let teacher_id = insert_user(&pool, "owner@example.com", "teacher").await;
    let other_teacher_id = insert_user(&pool, "other@example.com", "teacher").await;
    let student_id = insert_user(&pool, "student@example.com", "student").await;
    let course_id = insert_course(&pool, teacher_id).await;
    enroll_student(&pool, course_id, student_id).await;
    let activity_id = insert_activity(&pool, course_id, "published").await;

    let dir = temp_files_dir();
    let content = b"PK\x03\x04 test content";
    fs::write(dir.join("entrega.docx"), content).expect("test file is written");

    let delivery_id = insert_delivery(
        &pool,
        activity_id,
        student_id,
        "entrega.docx",
        "submitted",
        Utc::now(),
    )
    .await;
    let service = service(&pool, dir.clone());

    let bytes = service
        .download_delivery_file(teacher_id, &UserRole::Teacher, delivery_id)
        .await
        .expect("the owner teacher can download the file");
    assert_eq!(bytes, content);

    assert!(matches!(
        service
            .download_delivery_file(student_id, &UserRole::Student, delivery_id)
            .await,
        Err(DeliveryError::Forbidden)
    ));
    assert!(matches!(
        service
            .download_delivery_file(other_teacher_id, &UserRole::Teacher, delivery_id)
            .await,
        Err(DeliveryError::Activity(ActivityError::Forbidden))
    ));
    assert!(matches!(
        service
            .download_delivery_file(teacher_id, &UserRole::Teacher, Uuid::nil())
            .await,
        Err(DeliveryError::DeliveryNotFound)
    ));

    fs::remove_dir_all(dir).ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn download_fails_when_file_is_missing_or_path_is_unsafe(pool: PgPool) {
    let teacher_id = insert_user(&pool, "owner@example.com", "teacher").await;
    let first_student = insert_user(&pool, "first@example.com", "student").await;
    let second_student = insert_user(&pool, "second@example.com", "student").await;
    let course_id = insert_course(&pool, teacher_id).await;
    enroll_student(&pool, course_id, first_student).await;
    enroll_student(&pool, course_id, second_student).await;
    let activity_id = insert_activity(&pool, course_id, "published").await;

    let missing = insert_delivery(
        &pool,
        activity_id,
        first_student,
        "no-existe.docx",
        "submitted",
        Utc::now(),
    )
    .await;
    let unsafe_path = insert_delivery(
        &pool,
        activity_id,
        second_student,
        "../secreto.docx",
        "submitted",
        Utc::now(),
    )
    .await;
    let service = service(&pool, temp_files_dir());

    for delivery_id in [missing, unsafe_path] {
        assert!(matches!(
            service
                .download_delivery_file(teacher_id, &UserRole::Teacher, delivery_id)
                .await,
            Err(DeliveryError::FileUnavailable)
        ));
    }
}
