use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn persistence_enforces_unique_user_email_and_course_teacher_fk(pool: PgPool) {
    let first = sqlx::query(
        r#"
        INSERT INTO users (name, email, password, role, status, token)
        VALUES ('First', 'duplicate@example.com', 'hash', 'student', 'active', '')
        "#,
    )
    .execute(&pool)
    .await;
    assert!(first.is_ok());

    let duplicate = sqlx::query(
        r#"
        INSERT INTO users (name, email, password, role, status, token)
        VALUES ('Second', 'duplicate@example.com', 'hash', 'student', 'active', '')
        "#,
    )
    .execute(&pool)
    .await;
    assert!(duplicate.is_err());

    let invalid_course = sqlx::query(
        r#"
        INSERT INTO courses (id, name, code, teacher_id, password, status, quantity)
        VALUES ($1, 'Course', 'ABC123', $2, 'password', 'active', 10)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(Uuid::now_v7())
    .execute(&pool)
    .await;
    assert!(invalid_course.is_err());
}
