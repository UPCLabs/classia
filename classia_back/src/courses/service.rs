use uuid::Uuid;
use validator::Validate;

use crate::courses::{
    dto::CourseCreateDto, error::CourseError, models::Course, repository::CourseRepository,
};

pub struct CourseService {
    course_repositoy: CourseRepository,
}

impl CourseService {
    pub fn new(course_repositoy: CourseRepository) -> Self {
        Self { course_repositoy }
    }

    pub async fn create_course(&self, body: CourseCreateDto) -> Result<Course, CourseError> {
        let body = CourseCreateDto {
            name: body.name.trim().to_string(),
            code: body.code.trim().to_string(),
            password: body.password.trim().to_string(),
            status: body.status.trim().to_string(),
            ..body
        };

        body.validate().map_err(|errors| {
            let message = errors
                .field_errors()
                .values()
                .next()
                .and_then(|field_errors| field_errors.first())
                .and_then(|error| error.message.clone())
                .map(|message| message.to_string())
                .unwrap_or_else(|| "Datos invalidos".to_string());

            CourseError::ValidationError(message)
        })?;

        self.course_repositoy
            .create_course(body)
            .await
            .map_err(CourseError::Database)
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

    pub async fn get_course_by_code(&self, course_code: String) -> Result<Course, CourseError> {
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
