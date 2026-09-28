use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedUser,
    courses::dto::{CourseCreateDto, CourseResponseDto, CourseUpdateDto, EnrollStudentDto, StudentResponseDto},
    error::AppError,
    state::ClassiaState,
    users::UserRole,
};

#[derive(Debug, Deserialize)]
struct StudentSearchQuery {
    q: Option<String>,
}

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/", get(list_courses))
        .route("/{course_id}", get(get_course).patch(update_course))
        .route("/create", post(create_course))
        .route("/{course_id}/students", get(list_students).post(enroll_student))
        .route("/{course_id}/available-students", get(list_available_students))
}

async fn list_courses(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
) -> Result<Json<Value>, AppError> {
    let courses = state
        .course_service
        .list_for_user(user.id, &user.role)
        .await?;
    let items: Vec<CourseResponseDto> = courses.into_iter().map(Into::into).collect();

    Ok(Json(json!({ "items": items })))
}

async fn get_course(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
) -> Result<Json<CourseResponseDto>, AppError> {
    let course = state
        .course_service
        .get_for_user(course_id, user.id, &user.role)
        .await?;

    Ok(Json(course.into()))
}

async fn create_course(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Json(body): Json<CourseCreateDto>,
) -> Result<Json<CourseResponseDto>, AppError> {
    if !user.role.is_admin() && !matches!(user.role, UserRole::Teacher) {
        return Err(AppError::forbidden());
    }

    let course = state
        .course_service
        .create_course(user.id, &user.role, body)
        .await?;

    Ok(Json(course.into()))
}

async fn update_course(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
    Json(body): Json<CourseUpdateDto>,
) -> Result<Json<CourseResponseDto>, AppError> {
    if !user.role.is_admin() && !matches!(user.role, UserRole::Teacher) {
        return Err(AppError::forbidden());
    }

    let course = state
        .course_service
        .update_course(user.id, &user.role, course_id, body)
        .await?;

    Ok(Json(course.into()))
}

async fn list_students(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    require_teacher(&user)?;
    let students = state
        .enrollment_service
        .list_students(user.id, course_id)
        .await?;
    let items: Vec<StudentResponseDto> = students.into_iter().map(Into::into).collect();

    Ok(Json(json!({ "items": items })))
}

async fn list_available_students(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
    Query(query): Query<StudentSearchQuery>,
) -> Result<Json<Value>, AppError> {
    require_teacher(&user)?;
    let students = state
        .enrollment_service
        .list_available_students(user.id, course_id, query.q.as_deref())
        .await?;
    let items: Vec<StudentResponseDto> = students
        .into_iter()
        .map(|(id, name, email)| StudentResponseDto { id, name, email })
        .collect();

    Ok(Json(json!({ "items": items })))
}

async fn enroll_student(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
    Json(body): Json<EnrollStudentDto>,
) -> Result<StatusCode, AppError> {
    require_teacher(&user)?;
    state
        .enrollment_service
        .register_student(user.id, course_id, body)
        .await?;

    Ok(StatusCode::CREATED)
}

fn require_teacher(user: &AuthenticatedUser) -> Result<(), AppError> {
    if !matches!(user.role, UserRole::Teacher) {
        return Err(AppError::forbidden());
    }

    Ok(())
}
