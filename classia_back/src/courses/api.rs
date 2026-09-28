use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthenticatedUser,
    courses::dto::{
        CourseCreateDto, CourseResponseDto, CourseUpdateDto, EnrollStudentDto,
        EnrollmentResponseDto,
    },
    error::AppError,
    state::ClassiaState,
    users::{UserListResponseDto, UserResponseDto},
};

#[derive(Debug, Deserialize)]
struct StudentSearchQuery {
    q: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct CourseListResponseDto {
    items: Vec<CourseResponseDto>,
}

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/", get(list_courses).post(create_course))
        .route("/{course_id}", get(get_course).patch(update_course))
        .route(
            "/{course_id}/students",
            get(list_students).post(enroll_student),
        )
        .route(
            "/{course_id}/available-students",
            get(list_available_students),
        )
}

async fn list_courses(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
) -> Result<Json<CourseListResponseDto>, AppError> {
    let courses = state
        .course_service
        .list_for_user(user.id, &user.role)
        .await?;

    Ok(Json(CourseListResponseDto {
        items: courses.into_iter().map(Into::into).collect(),
    }))
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
) -> Result<(StatusCode, Json<CourseResponseDto>), AppError> {
    let course = state
        .course_service
        .create_course(user.id, &user.role, body)
        .await?;

    Ok((StatusCode::CREATED, Json(course.into())))
}

async fn update_course(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
    Json(body): Json<CourseUpdateDto>,
) -> Result<Json<CourseResponseDto>, AppError> {
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
) -> Result<Json<UserListResponseDto>, AppError> {
    let students = state
        .enrollment_service
        .list_students(user.id, &user.role, course_id)
        .await?;

    Ok(Json(user_list(students)))
}

async fn list_available_students(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
    Query(query): Query<StudentSearchQuery>,
) -> Result<Json<UserListResponseDto>, AppError> {
    let students = state
        .enrollment_service
        .list_available_students(user.id, &user.role, course_id, query.q.as_deref())
        .await?;

    Ok(Json(user_list(students)))
}

async fn enroll_student(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
    Json(body): Json<EnrollStudentDto>,
) -> Result<(StatusCode, Json<EnrollmentResponseDto>), AppError> {
    let enrollment = state
        .enrollment_service
        .enroll_student(user.id, &user.role, course_id, body)
        .await?;

    Ok((StatusCode::CREATED, Json(enrollment.into())))
}

fn user_list(users: Vec<crate::users::User>) -> UserListResponseDto {
    UserListResponseDto {
        items: users.into_iter().map(UserResponseDto::from).collect(),
    }
}
