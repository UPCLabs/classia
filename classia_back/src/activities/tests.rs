use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{build_service, error::ActivityError, models::ActivityStatus};
use crate::{
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
    sqlx::query(
        "INSERT INTO course_enrollments (course_id, student_id)
         VALUES ($1, $2)",
    )
    .bind(course_id)
    .bind(student_id)
    .execute(pool)
    .await
    .expect("test student is enrolled");
}

async fn insert_activity(
    pool: &PgPool,
    course_id: Uuid,
    title: &str,
    due_at: DateTime<Utc>,
    status: &str,
) -> Uuid {
    let published_at = (status == "published").then(Utc::now);

    sqlx::query_scalar(
        "INSERT INTO activities
            (course_id, title, description, due_at, status, published_at)
         VALUES ($1, $2, 'Test description', $3, $4::activity_status, $5)
         RETURNING id",
    )
    .bind(course_id)
    .bind(title)
    .bind(due_at)
    .bind(status)
    .bind(published_at)
    .fetch_one(pool)
    .await
    .expect("test activity is inserted")
}

fn service(pool: &PgPool) -> super::ActivityService {
    let user_service = Arc::new(users::build_service(pool.clone()));
    let course_service = Arc::new(courses::build_service(pool.clone(), user_service));

    build_service(pool.clone(), course_service)
}

#[test]
fn activity_errors_map_to_contract_codes() {
    let cases = [
        (
            ActivityError::NoChanges,
            StatusCode::BAD_REQUEST,
            "NO_CHANGES",
        ),
        (
            ActivityError::AlreadyPublished,
            StatusCode::CONFLICT,
            "ACTIVITY_ALREADY_PUBLISHED",
        ),
        (
            ActivityError::ActivityNotFound,
            StatusCode::NOT_FOUND,
            "ACTIVITY_NOT_FOUND",
        ),
        (ActivityError::Forbidden, StatusCode::FORBIDDEN, "FORBIDDEN"),
    ];

    for (error, status, code) in cases {
        let mapped: AppError = error.into();
        assert_eq!(mapped.status, status, "{code}");
        assert_eq!(mapped.code, code);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn lists_activities_according_to_course_role(pool: PgPool) {
    let teacher_id = insert_user(&pool, "owner@example.com", "teacher").await;
    let other_teacher_id = insert_user(&pool, "other@example.com", "teacher").await;
    let admin_id = insert_user(&pool, "admin@example.com", "admin").await;
    let super_admin_id = insert_user(&pool, "super@example.com", "super_admin").await;
    let student_id = insert_user(&pool, "student@example.com", "student").await;
    let course_id = insert_course(&pool, teacher_id).await;
    enroll_student(&pool, course_id, student_id).await;

    let draft_id = insert_activity(
        &pool,
        course_id,
        "Earlier draft",
        Utc::now() + Duration::days(1),
        "draft",
    )
    .await;
    let published_id = insert_activity(
        &pool,
        course_id,
        "Later published",
        Utc::now() + Duration::days(2),
        "published",
    )
    .await;
    let service = service(&pool);

    let teacher_activities = service
        .list_for_user(teacher_id, &UserRole::Teacher, course_id)
        .await
        .expect("the owner teacher can list all activities");
    assert_eq!(
        teacher_activities
            .iter()
            .map(|activity| activity.id)
            .collect::<Vec<_>>(),
        vec![draft_id, published_id]
    );

    for (user_id, role) in [
        (admin_id, UserRole::Admin),
        (super_admin_id, UserRole::SuperAdmin),
    ] {
        let activities = service
            .list_for_user(user_id, &role, course_id)
            .await
            .expect("administrators can list all activities");
        assert_eq!(activities.len(), 2);
    }

    let student_activities = service
        .list_for_user(student_id, &UserRole::Student, course_id)
        .await
        .expect("an enrolled student can list published activities");
    assert_eq!(student_activities.len(), 1);
    assert_eq!(student_activities[0].id, published_id);
    assert_eq!(student_activities[0].status, ActivityStatus::Published);

    assert!(matches!(
        service
            .list_for_user(other_teacher_id, &UserRole::Teacher, course_id)
            .await,
        Err(ActivityError::Forbidden)
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn gets_activity_detail_according_to_course_role(pool: PgPool) {
    let teacher_id = insert_user(&pool, "owner@example.com", "teacher").await;
    let other_teacher_id = insert_user(&pool, "other@example.com", "teacher").await;
    let admin_id = insert_user(&pool, "admin@example.com", "admin").await;
    let super_admin_id = insert_user(&pool, "super@example.com", "super_admin").await;
    let student_id = insert_user(&pool, "student@example.com", "student").await;
    let course_id = insert_course(&pool, teacher_id).await;
    enroll_student(&pool, course_id, student_id).await;

    let draft_id = insert_activity(
        &pool,
        course_id,
        "Draft",
        Utc::now() + Duration::days(1),
        "draft",
    )
    .await;
    let published_id = insert_activity(
        &pool,
        course_id,
        "Published",
        Utc::now() + Duration::days(2),
        "published",
    )
    .await;
    let service = service(&pool);

    for (user_id, role) in [
        (teacher_id, UserRole::Teacher),
        (admin_id, UserRole::Admin),
        (super_admin_id, UserRole::SuperAdmin),
    ] {
        let activity = service
            .get_for_user(user_id, &role, draft_id)
            .await
            .expect("authorized staff can view draft details");
        assert_eq!(activity.id, draft_id);
    }

    let published = service
        .get_for_user(student_id, &UserRole::Student, published_id)
        .await
        .expect("an enrolled student can view a published activity");
    assert_eq!(published.id, published_id);

    assert!(matches!(
        service
            .get_for_user(student_id, &UserRole::Student, draft_id)
            .await,
        Err(ActivityError::ActivityNotFound)
    ));
    assert!(matches!(
        service
            .get_for_user(other_teacher_id, &UserRole::Teacher, draft_id)
            .await,
        Err(ActivityError::Forbidden)
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn publishes_activity_successfully_and_validates_due_date(pool: PgPool) {
    let teacher_id = insert_user(&pool, "teacher_pub@example.com", "teacher").await;
    let course_id = insert_course(&pool, teacher_id).await;
    let service = service(&pool);

    let valid_draft_id = insert_activity(
        &pool,
        course_id,
        "Valid Draft",
        Utc::now() + Duration::days(1),
        "draft",
    )
    .await;

    let result = service
        .publish(teacher_id, &UserRole::Teacher, valid_draft_id)
        .await;
    assert!(
        result.is_ok(),
        "The activity should be published successfully"
    );
    assert_eq!(result.unwrap().status, ActivityStatus::Published);

    let expired_draft_id = insert_activity(
        &pool,
        course_id,
        "Expired Draft",
        Utc::now() - Duration::days(1),
        "draft",
    )
    .await;

    let result_expired = service
        .publish(teacher_id, &UserRole::Teacher, expired_draft_id)
        .await;

    assert!(
        result_expired.is_err(),
        "Should not publish if the due date has already passed"
    );

    let published_id = insert_activity(
        &pool,
        course_id,
        "Already Published",
        Utc::now() + Duration::days(1),
        "published",
    )
    .await;

    let result_already_pub = service
        .publish(teacher_id, &UserRole::Teacher, published_id)
        .await;
    assert!(matches!(
        result_already_pub,
        Err(ActivityError::AlreadyPublished)
    ));
}
