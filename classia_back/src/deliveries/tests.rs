use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{Delivery, DeliveryStatus, Evaluation};

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

async fn insert_published_activity(pool: &PgPool, teacher_id: Uuid) -> Uuid {
    let course_id: Uuid = sqlx::query_scalar(
        "INSERT INTO courses (name, code, teacher_id, status, capacity)
         VALUES ('Software Engineering', 'IS212', $1, 'active', 20)
         RETURNING id",
    )
    .bind(teacher_id)
    .fetch_one(pool)
    .await
    .expect("test course is inserted");

    sqlx::query_scalar(
        "INSERT INTO activities
            (course_id, title, description, due_at, status, published_at)
         VALUES ($1, 'Workshop', 'Description', $2, 'published', now())
         RETURNING id",
    )
    .bind(course_id)
    .bind(Utc::now() + Duration::days(1))
    .fetch_one(pool)
    .await
    .expect("test activity is inserted")
}

async fn insert_delivery(
    pool: &PgPool,
    activity_id: Uuid,
    student_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO deliveries (activity_id, student_id, file_url)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(activity_id)
    .bind(student_id)
    .bind(format!("{}.docx", Uuid::now_v7()))
    .fetch_one(pool)
    .await
}

async fn insert_evaluation(
    pool: &PgPool,
    delivery_id: Uuid,
    teacher_id: Uuid,
    score: f64,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO evaluations (delivery_id, score, feedback, teacher_id)
         VALUES ($1, $2::numeric, 'Feedback', $3)
         RETURNING id",
    )
    .bind(delivery_id)
    .bind(score)
    .bind(teacher_id)
    .fetch_one(pool)
    .await
}

struct Fixture {
    teacher_id: Uuid,
    student_id: Uuid,
    activity_id: Uuid,
}

async fn fixture(pool: &PgPool) -> Fixture {
    let teacher_id = insert_user(pool, "teacher@example.com", "teacher").await;
    let student_id = insert_user(pool, "student@example.com", "student").await;
    let activity_id = insert_published_activity(pool, teacher_id).await;

    Fixture {
        teacher_id,
        student_id,
        activity_id,
    }
}

fn is_check_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db_error) if db_error.is_check_violation())
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db_error) if db_error.is_unique_violation())
}

#[sqlx::test(migrations = "./migrations")]
async fn new_delivery_defaults_to_submitted_version_one(pool: PgPool) {
    let f = fixture(&pool).await;
    let delivery_id = insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect("first delivery is inserted");

    let delivery: Delivery = sqlx::query_as(
        "SELECT id, activity_id, student_id, file_url, status, version,
                submitted_at, updated_at
         FROM deliveries WHERE id = $1",
    )
    .bind(delivery_id)
    .fetch_one(&pool)
    .await
    .expect("delivery decodes into the model");

    assert_eq!(delivery.activity_id, f.activity_id);
    assert_eq!(delivery.student_id, f.student_id);
    assert_eq!(delivery.status, DeliveryStatus::Submitted);
    assert_eq!(delivery.version, 1);
    assert!(delivery.file_url.ends_with(".docx"));
}

#[sqlx::test(migrations = "./migrations")]
async fn second_delivery_for_same_student_and_activity_is_rejected(pool: PgPool) {
    let f = fixture(&pool).await;
    insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect("first delivery is inserted");

    let duplicate = insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect_err("the unique index rejects a second delivery");
    assert!(is_unique_violation(&duplicate));

    let constraint = match &duplicate {
        sqlx::Error::Database(db_error) => db_error.constraint().map(str::to_owned),
        _ => None,
    };
    assert_eq!(
        constraint.as_deref(),
        Some("deliveries_activity_student_uk")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn other_students_can_deliver_the_same_activity(pool: PgPool) {
    let f = fixture(&pool).await;
    let other_student_id = insert_user(&pool, "other@example.com", "student").await;

    insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect("first student delivers");
    insert_delivery(&pool, f.activity_id, other_student_id)
        .await
        .expect("a different student delivers the same activity");
}

#[sqlx::test(migrations = "./migrations")]
async fn delivery_version_cannot_drop_below_one(pool: PgPool) {
    let f = fixture(&pool).await;
    let delivery_id = insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect("delivery is inserted");

    let error = sqlx::query("UPDATE deliveries SET version = 0 WHERE id = $1")
        .bind(delivery_id)
        .execute(&pool)
        .await
        .expect_err("version below one violates the check");
    assert!(is_check_violation(&error));
}

#[sqlx::test(migrations = "./migrations")]
async fn evaluation_score_must_be_between_zero_and_five(pool: PgPool) {
    let f = fixture(&pool).await;

    for (index, score) in [-0.1, 5.1].into_iter().enumerate() {
        let student_id = insert_user(&pool, &format!("s{index}@example.com"), "student").await;
        let delivery_id = insert_delivery(&pool, f.activity_id, student_id)
            .await
            .expect("delivery is inserted");

        let error = insert_evaluation(&pool, delivery_id, f.teacher_id, score)
            .await
            .expect_err("out-of-range score is rejected");
        assert!(is_check_violation(&error), "score {score}");
    }

    for (index, score) in [0.0, 5.0].into_iter().enumerate() {
        let student_id = insert_user(&pool, &format!("ok{index}@example.com"), "student").await;
        let delivery_id = insert_delivery(&pool, f.activity_id, student_id)
            .await
            .expect("delivery is inserted");

        insert_evaluation(&pool, delivery_id, f.teacher_id, score)
            .await
            .unwrap_or_else(|error| panic!("boundary score {score} is accepted: {error}"));
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn evaluation_score_is_rounded_to_one_decimal(pool: PgPool) {
    let f = fixture(&pool).await;
    let delivery_id = insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect("delivery is inserted");
    let evaluation_id = insert_evaluation(&pool, delivery_id, f.teacher_id, 4.25)
        .await
        .expect("evaluation is inserted");

    let evaluation: Evaluation = sqlx::query_as(
        "SELECT id, delivery_id, score::float8 AS score, feedback, teacher_id,
                graded_at, updated_at
         FROM evaluations WHERE id = $1",
    )
    .bind(evaluation_id)
    .fetch_one(&pool)
    .await
    .expect("evaluation decodes into the model");

    assert_eq!(evaluation.delivery_id, delivery_id);
    assert_eq!(evaluation.teacher_id, f.teacher_id);
    assert_eq!(evaluation.score, 4.3);
    assert_eq!(evaluation.feedback.as_deref(), Some("Feedback"));
}

#[sqlx::test(migrations = "./migrations")]
async fn delivery_accepts_only_one_evaluation(pool: PgPool) {
    let f = fixture(&pool).await;
    let delivery_id = insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect("delivery is inserted");

    insert_evaluation(&pool, delivery_id, f.teacher_id, 4.0)
        .await
        .expect("first evaluation is inserted");
    let duplicate = insert_evaluation(&pool, delivery_id, f.teacher_id, 3.0)
        .await
        .expect_err("a second evaluation for the same delivery is rejected");
    assert!(is_unique_violation(&duplicate));
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_a_delivery_removes_its_evaluation(pool: PgPool) {
    let f = fixture(&pool).await;
    let delivery_id = insert_delivery(&pool, f.activity_id, f.student_id)
        .await
        .expect("delivery is inserted");
    insert_evaluation(&pool, delivery_id, f.teacher_id, 4.0)
        .await
        .expect("evaluation is inserted");

    sqlx::query("DELETE FROM deliveries WHERE id = $1")
        .bind(delivery_id)
        .execute(&pool)
        .await
        .expect("delivery is deleted");

    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM evaluations WHERE delivery_id = $1")
            .bind(delivery_id)
            .fetch_one(&pool)
            .await
            .expect("evaluations are counted");
    assert_eq!(remaining, 0);
}
