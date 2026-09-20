use uuid::Uuid;

use crate::users::{models::Course, repository::CourseRepository};

pub struct CourseService {
    course_repositoy: CourseRepository,
}

impl CourseService {
    pub fn new(course_repositoy: CourseRepository) -> Self {
        Self { course_repositoy }
    }

    // We need to change to our custom errors... API must not know about postgres
    // Here is an example, but we need to know, what way is better
    pub async fn get_course_by_id(&self, id_course: Uuid) -> Result<Course, CourseError> {
        self.course_repositoy
            .get_course_by_id(id_course)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => CourseError::CourseNotFound,
                error => CourseError::Database(error),
            })
    }

    pub async fn get_course_by_code(&self, course_code: Uuid) -> Result<Course, CourseError> {
        self.course_repositoy
            .get_course_by_code(course_code)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => CourseError::CourseNotFound,
                error => CourseError::Database(error),
            })
    }

    pub async fn get_courses_by_teacher(&self, teacher_id: Uuid) -> Result<Course, CourseError> {
        self.course_repositoy
            .get_courses_by_teacher(teacher_id)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => CourseError::CourseNotFound,
                error => CourseError::Database(error),
            })
    }

   
}
