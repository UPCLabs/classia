use axum::http::StatusCode;

use crate::error::AppError;

#[derive(Debug)]
pub enum CourseError {
    CourseNotFound,
    TeacherNotFound,
    TeacherInactive,
    InvalidTeacherRole,
    Forbidden,
    DuplicateCode,
    CapacityBelowEnrollment,
    ValidationError(String),
    Database(sqlx::Error),
}

#[derive(Debug)]
pub enum EnrollmentError {
    CourseNotFound,
    CourseNotActive,
    CourseFull,
    StudentNotFound,
    StudentMustHaveStudentRole,
    StudentInactive,
    TeacherDoesNotOwnCourse,
    AlreadyEnrolled,
    Database(sqlx::Error),
}

impl From<CourseError> for AppError {
    fn from(value: CourseError) -> Self {
        match value {
            CourseError::CourseNotFound => error(StatusCode::NOT_FOUND, "COURSE_NOT_FOUND", "Curso no encontrado"),
            CourseError::TeacherNotFound => error(StatusCode::NOT_FOUND, "TEACHER_NOT_FOUND", "Docente no encontrado"),
            CourseError::TeacherInactive => error(StatusCode::CONFLICT, "TEACHER_INACTIVE", "El docente está inactivo"),
            CourseError::InvalidTeacherRole => error(StatusCode::BAD_REQUEST, "INVALID_TEACHER_ROLE", "El usuario no tiene rol de docente"),
            CourseError::Forbidden => AppError::forbidden(),
            CourseError::DuplicateCode => error(StatusCode::CONFLICT, "COURSE_CODE_EXISTS", "El código del curso ya existe"),
            CourseError::CapacityBelowEnrollment => error(StatusCode::CONFLICT, "CAPACITY_BELOW_ENROLLMENT", "La capacidad no puede ser menor que los estudiantes matriculados"),
            CourseError::ValidationError(message) => error(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &message),
            CourseError::Database(error) => database_error(error),
        }
    }
}

impl From<EnrollmentError> for AppError {
    fn from(value: EnrollmentError) -> Self {
        match value {
            EnrollmentError::CourseNotFound => error(StatusCode::NOT_FOUND, "COURSE_NOT_FOUND", "Curso no encontrado"),
            EnrollmentError::CourseNotActive => error(StatusCode::CONFLICT, "COURSE_NOT_ACTIVE", "El curso no está activo"),
            EnrollmentError::CourseFull => error(StatusCode::CONFLICT, "COURSE_FULL", "El curso no tiene cupos disponibles"),
            EnrollmentError::StudentNotFound => error(StatusCode::NOT_FOUND, "STUDENT_NOT_FOUND", "Estudiante no encontrado"),
            EnrollmentError::StudentMustHaveStudentRole => error(StatusCode::BAD_REQUEST, "INVALID_STUDENT_ROLE", "El usuario no tiene rol de estudiante"),
            EnrollmentError::StudentInactive => error(StatusCode::CONFLICT, "STUDENT_INACTIVE", "El estudiante está inactivo"),
            EnrollmentError::TeacherDoesNotOwnCourse => error(StatusCode::FORBIDDEN, "COURSE_ACCESS_FORBIDDEN", "El docente no es dueño del curso"),
            EnrollmentError::AlreadyEnrolled => error(StatusCode::CONFLICT, "ALREADY_ENROLLED", "El estudiante ya está matriculado"),
            EnrollmentError::Database(error) => database_error(error),
        }
    }
}

fn error(status: StatusCode, code: &'static str, message: &str) -> AppError {
    AppError {
        status,
        code,
        message: message.to_string(),
    }
}

fn database_error(database: sqlx::Error) -> AppError {
    eprintln!("Database error: {database}");
    error(StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error")
}
