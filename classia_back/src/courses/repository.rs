use chrono::{DateTime, Utc};
use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::courses::{
    dto::{CourseCreateDto, CourseUpdateDto},
    models::Course,
};
use crate::users::User;

const COURSE_SELECT: &str = r#"
    SELECT
        c.id,
        c.name,
        c.code,
        c.teacher_id,
        teacher.name AS teacher_name,
        teacher.email AS teacher_email,
        c.status,
        c.capacity,
        COUNT(enrollment.student_id)::BIGINT AS enrolled_count,
        c.created_at,
        c.updated_at
    FROM courses c
    JOIN users teacher ON teacher.id = c.teacher_id
    LEFT JOIN course_enrollments enrollment ON enrollment.course_id = c.id
"#;

const USER_COLUMNS: &str =
    "u.id, u.name, u.email, u.password, u.role, u.status, u.token, u.created_at, u.updated_at";

pub enum EnrollOutcome {
    Enrolled(DateTime<Utc>),
    AlreadyEnrolled,
    CourseFull,
}

pub struct CourseRepository {
    pool: PgPool,
}

impl CourseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_all(&self) -> Result<Vec<Course>, sqlx::Error> {
        query_as::<_, Course>(&format!(
            "{COURSE_SELECT} GROUP BY c.id, teacher.id ORDER BY c.created_at DESC"
        ))
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_by_teacher(&self, teacher_id: Uuid) -> Result<Vec<Course>, sqlx::Error> {
        query_as::<_, Course>(&format!(
            "{COURSE_SELECT} WHERE c.teacher_id = $1 GROUP BY c.id, teacher.id ORDER BY c.created_at DESC"
        ))
        .bind(teacher_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_by_student(&self, student_id: Uuid) -> Result<Vec<Course>, sqlx::Error> {
        query_as::<_, Course>(&format!(
            "{COURSE_SELECT}
             WHERE EXISTS (
                 SELECT 1 FROM course_enrollments student_enrollment
                 WHERE student_enrollment.course_id = c.id
                   AND student_enrollment.student_id = $1
             )
             GROUP BY c.id, teacher.id ORDER BY c.created_at DESC"
        ))
        .bind(student_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_id(&self, course_id: Uuid) -> Result<Course, sqlx::Error> {
        query_as::<_, Course>(&format!(
            "{COURSE_SELECT} WHERE c.id = $1 GROUP BY c.id, teacher.id"
        ))
        .bind(course_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create(&self, body: CourseCreateDto) -> Result<Course, sqlx::Error> {
        let id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO courses (name, code, teacher_id, status, capacity)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
        )
        .bind(body.name)
        .bind(body.code)
        .bind(body.teacher_id)
        .bind(body.status)
        .bind(body.capacity)
        .fetch_one(&self.pool)
        .await?;

        self.get_by_id(id).await
    }

    pub async fn update(
        &self,
        course_id: Uuid,
        body: CourseUpdateDto,
    ) -> Result<Course, sqlx::Error> {
        sqlx::query(
            "UPDATE courses
             SET name = COALESCE($1, name),
                 code = COALESCE($2, code),
                 teacher_id = COALESCE($3, teacher_id),
                 status = COALESCE($4, status),
                 capacity = COALESCE($5, capacity),
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $6",
        )
        .bind(body.name)
        .bind(body.code)
        .bind(body.teacher_id)
        .bind(body.status)
        .bind(body.capacity)
        .bind(course_id)
        .execute(&self.pool)
        .await?;

        self.get_by_id(course_id).await
    }

    pub async fn list_enrolled_students(&self, course_id: Uuid) -> Result<Vec<User>, sqlx::Error> {
        query_as::<_, User>(&format!(
            "SELECT {USER_COLUMNS}
             FROM course_enrollments e
             JOIN users u ON u.id = e.student_id
             WHERE e.course_id = $1
             ORDER BY u.name"
        ))
        .bind(course_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_available_students(
        &self,
        course_id: Uuid,
        search: Option<&str>,
    ) -> Result<Vec<User>, sqlx::Error> {
        query_as::<_, User>(&format!(
            "SELECT {USER_COLUMNS}
             FROM users u
             WHERE u.role = 'student'
               AND u.status = 'active'
               AND ($2::text IS NULL OR u.name ILIKE '%' || $2 || '%' OR u.email ILIKE '%' || $2 || '%')
               AND NOT EXISTS (
                   SELECT 1 FROM course_enrollments e
                   WHERE e.course_id = $1 AND e.student_id = u.id
               )
             ORDER BY u.name"
        ))
        .bind(course_id)
        .bind(search)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn enroll_student(
        &self,
        course_id: Uuid,
        student_id: Uuid,
    ) -> Result<EnrollOutcome, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        let capacity =
            sqlx::query_scalar::<_, i16>("SELECT capacity FROM courses WHERE id = $1 FOR UPDATE")
                .bind(course_id)
                .fetch_one(&mut *transaction)
                .await?;

        let already_enrolled = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                 SELECT 1 FROM course_enrollments
                 WHERE course_id = $1 AND student_id = $2
             )",
        )
        .bind(course_id)
        .bind(student_id)
        .fetch_one(&mut *transaction)
        .await?;

        if already_enrolled {
            return Ok(EnrollOutcome::AlreadyEnrolled);
        }

        let enrolled = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM course_enrollments WHERE course_id = $1",
        )
        .bind(course_id)
        .fetch_one(&mut *transaction)
        .await?;

        if enrolled >= i64::from(capacity) {
            return Ok(EnrollOutcome::CourseFull);
        }

        let enrolled_at = sqlx::query_scalar::<_, DateTime<Utc>>(
            "INSERT INTO course_enrollments (course_id, student_id)
             VALUES ($1, $2)
             RETURNING enrolled_at",
        )
        .bind(course_id)
        .bind(student_id)
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(EnrollOutcome::Enrolled(enrolled_at))
    }

    pub async fn is_enrolled(
        &self,
        course_id: Uuid,
        student_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT EXISTS(
                 SELECT 1 FROM course_enrollments
                 WHERE course_id = $1 AND student_id = $2
             )",
        )
        .bind(course_id)
        .bind(student_id)
        .fetch_one(&self.pool)
        .await
    }
}
