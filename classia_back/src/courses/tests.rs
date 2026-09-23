use axum::http::StatusCode;
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use super::{
    dto::{CourseCreateDto, CourseResponseDto},
    error::CourseError,
    models::Course,
};
use crate::error::AppError;

fn valid_course() -> CourseCreateDto {
    CourseCreateDto {
        name: "Software Engineering".to_string(),
        code: "IS212".to_string(),
        teacher_id: Uuid::now_v7(),
        password: "course-password".to_string(),
        status: "active".to_string(),
        quantity: 20,
    }
}

#[test]
fn validates_course_fields() {
    assert!(valid_course().validate().is_ok());

    let mut invalid = valid_course();
    invalid.code = "TOO-LONG".to_string();
    invalid.quantity = -1;

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
        password: "course-password".to_string(),
        status: "active".to_string(),
        quantity: 20,
        created_at: now,
        updated_at: now,
    };

    let response = CourseResponseDto::from(course);

    assert_eq!(response.name, "Software Engineering");
    assert_eq!(response.code, "IS212");
    assert_eq!(response.quantity, 20);
}

#[test]
fn course_errors_map_to_stable_http_statuses() {
    let not_found: AppError = CourseError::CourseNotFound.into();
    let invalid: AppError = CourseError::ValidationError("invalid".to_string()).into();

    assert_eq!(not_found.status, StatusCode::NOT_FOUND);
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
}
