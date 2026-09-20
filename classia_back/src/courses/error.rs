#[derive(Debug)]
pub enum CourseError {
    CourseNotFound,
    Database(sqlx::Error),
}

impl From<CourseError> for String {
    fn from(value: CourseError) -> Self {
        match value {
            CourseError::CourseNotFound => format!("Curso no encontrado"),
            CourseError::Database(error) => format!("Database error: {}", error),
        }
    }
}
