use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use uuid::Uuid;

use crate::{
    courses::dto::{CourseCreateDto, CourseResponseDto},
    error::AppError,
    state::ClassiaState,
};

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/ById/{id_course}", get(get_course_by_id))
        .route("/getCourseByCode/{course_code}", get(get_course_by_code))
        .route(
            "/getTeachersCourse/{teacher_id}",
            get(get_courses_by_teacher),
        )
        .route("/create", post(create_course))
}

async fn get_course_by_id(
    State(state): State<ClassiaState>,
    Path(id_course): Path<Uuid>,
) -> Result<Json<CourseResponseDto>, AppError> {
    let course = state.course_service.get_course_by_id(id_course).await?;

    Ok(Json(course.into()))
}

async fn get_course_by_code(
    State(state): State<ClassiaState>,
    Path(course_code): Path<String>,
) -> Result<Json<CourseResponseDto>, AppError> {
    let course = state.course_service.get_course_by_code(course_code).await?;

    Ok(Json(course.into()))
}

async fn get_courses_by_teacher(
    State(state): State<ClassiaState>,
    Path(teacher_id): Path<Uuid>,
) -> Result<Json<CourseResponseDto>, AppError> {
    let course = state
        .course_service
        .get_courses_by_teacher(teacher_id)
        .await?;

    Ok(Json(course.into()))
}

async fn create_course(
    State(state): State<ClassiaState>,
    Json(body): Json<CourseCreateDto>,
) -> Result<Json<CourseResponseDto>, AppError> {
    let course = state.course_service.create_course(body).await?;

    Ok(Json(course.into()))
}
