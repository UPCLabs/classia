use axum::http::StatusCode;
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use super::{
    dto::{CourseCreateDto, CourseResponseDto},
    error::CourseError,
    models::{Course, CourseStatus},
};
use crate::error::AppError;

fn valid_course() -> CourseCreateDto {
    CourseCreateDto {
        name: "Software Engineering".to_string(),
        code: "IS212".to_string(),
        teacher_id: Uuid::now_v7(),
        status: CourseStatus::Active,
        capacity: 20,
    }
}

#[test]
fn validates_course_fields() {
    assert!(valid_course().validate().is_ok());

    let mut invalid = valid_course();
    invalid.code = "TOO-LONG".to_string();
    invalid.capacity = 0;

    assert!(invalid.validate().is_err());
}

#[test]
fn course_response_copies_persisted_fields() {
    let now = Utc::now();
    let course = Course {
        id: Uuid::now_v7(),
        name: "Software Engineering".to_string(),
        code: "IS212".to_string(),
        teacher_id: Uuid::now_v7(),
        teacher_name: "Teacher".to_string(),
        teacher_email: "teacher@example.com".to_string(),
        status: CourseStatus::Active,
        capacity: 20,
        enrolled_count: 0,
        created_at: now,
        updated_at: now,
    };

    let response = CourseResponseDto::from(course);

    assert_eq!(response.name, "Software Engineering");
    assert_eq!(response.code, "IS212");
    assert_eq!(response.capacity, 20);
    assert_eq!(response.teacher.name, "Teacher");
}

#[test]
fn course_errors_map_to_stable_http_statuses() {
    let not_found: AppError = CourseError::CourseNotFound.into();
    let invalid: AppError = CourseError::ValidationError("invalid".to_string()).into();

    assert_eq!(not_found.status, StatusCode::NOT_FOUND);
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
}
