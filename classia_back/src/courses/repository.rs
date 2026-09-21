use sqlx::PgPool;
use sqlx::query_as;
use uuid::Uuid;

use crate::courses::models::Course;

pub struct CourseRepository {
    pool: PgPool,
}

impl CourseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_course_by_id(&self, id_course: Uuid) -> Result<Course, sqlx::Error> {
        let course = query_as::<_, Course>(
            r#"
              SELECT
                  id,
                  name,
                  code,
                  teacher_id,
                  password,
                  status,
                  quantity,
                  created_at,
                  updated_at
              FROM courses
              WHERE id = $1
            "#,
        )
        .bind(id_course)
        .fetch_one(&self.pool)
        .await?;

        Ok(course)
    }

    pub async fn get_course_by_code(&self, course_code: String) -> Result<Course, sqlx::Error> {
        let course = query_as::<_, Course>(
            r#"
              SELECT
                  id,
                  name,
                  code,
                  teacher_id,
                  password,
                  status,
                  quantity,
                  created_at,
                  updated_at
              FROM courses
              WHERE code = $1
            "#,
        )
        .bind(course_code)
        .fetch_one(&self.pool)
        .await?;

        Ok(course)
    }

    pub async fn get_courses_by_teacher(&self, teacher_id: Uuid) -> Result<Course, sqlx::Error> {
        let courses = query_as::<_, Course>(
            r#"
              SELECT
                  id,
                  name,
                  code,
                  teacher_id,
                  password,
                  status,
                  quantity,
                  created_at,
                  updated_at
              FROM courses
              WHERE teacher_id = $1
            "#,
        )
        .bind(teacher_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(courses)
    }
}
