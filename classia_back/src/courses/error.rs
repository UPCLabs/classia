use axum::http::StatusCode;
use tracing::error;

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
    InvalidStudentRole,
    StudentInactive,
    Forbidden,
    AlreadyEnrolled,
    Database(sqlx::Error),
}

impl From<CourseError> for AppError {
    fn from(value: CourseError) -> Self {
        match value {
            CourseError::CourseNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "COURSE_NOT_FOUND",
                "Curso no encontrado",
            ),
            CourseError::TeacherNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "TEACHER_NOT_FOUND",
                "Docente no encontrado",
            ),
            CourseError::TeacherInactive => AppError::new(
                StatusCode::CONFLICT,
                "TEACHER_INACTIVE",
                "El docente está inactivo",
            ),
            CourseError::InvalidTeacherRole => AppError::new(
                StatusCode::BAD_REQUEST,
                "INVALID_TEACHER_ROLE",
                "El usuario no tiene rol de docente",
            ),
            CourseError::Forbidden => AppError::forbidden(),
            CourseError::DuplicateCode => AppError::new(
                StatusCode::CONFLICT,
                "COURSE_CODE_EXISTS",
                "El código del curso ya existe",
            ),
            CourseError::CapacityBelowEnrollment => AppError::new(
                StatusCode::CONFLICT,
                "CAPACITY_BELOW_ENROLLMENT",
                "El cupo no puede ser menor que los estudiantes inscritos",
            ),
            CourseError::ValidationError(message) => {
                AppError::new(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message)
            }
            CourseError::Database(error) => database_error(error),
        }
    }
}

impl From<EnrollmentError> for AppError {
    fn from(value: EnrollmentError) -> Self {
        match value {
            EnrollmentError::CourseNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "COURSE_NOT_FOUND",
                "Curso no encontrado",
            ),
            EnrollmentError::CourseNotActive => AppError::new(
                StatusCode::CONFLICT,
                "COURSE_NOT_ACTIVE",
                "El curso no está activo",
            ),
            EnrollmentError::CourseFull => AppError::new(
                StatusCode::CONFLICT,
                "COURSE_CAPACITY_REACHED",
                "El curso no tiene cupos disponibles",
            ),
            EnrollmentError::StudentNotFound => AppError::new(
                StatusCode::NOT_FOUND,
                "STUDENT_NOT_FOUND",
                "Estudiante no encontrado",
            ),
            EnrollmentError::InvalidStudentRole => AppError::new(
                StatusCode::BAD_REQUEST,
                "INVALID_STUDENT_ROLE",
                "El usuario no tiene rol de estudiante",
            ),
            EnrollmentError::StudentInactive => AppError::new(
                StatusCode::CONFLICT,
                "STUDENT_INACTIVE",
                "El estudiante está inactivo",
            ),
            EnrollmentError::Forbidden => AppError::forbidden(),
            EnrollmentError::AlreadyEnrolled => AppError::new(
                StatusCode::CONFLICT,
                "STUDENT_ALREADY_ENROLLED",
                "El estudiante ya está inscrito en el curso",
            ),
            EnrollmentError::Database(error) => database_error(error),
        }
    }
}

fn database_error(database: sqlx::Error) -> AppError {
    error!("Database error: {database}");
    AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "DATABASE_ERROR",
        "Error de base de datos",
    )
}
