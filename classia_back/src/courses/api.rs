use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::users::{dto::CourseResponseDto, repository::CourseRepository, service::CourseService};

pub fn route(pool: PgPool) -> Router {
    let repository = CourseRepository::new(pool);
    let service = Arc::new(CourseService::new(repository));

    Router::new()
        .route("/courses/getCourseById/{id_course}", get(get_course_by_id))
        .route("/courses/getCourseByCode/{course_code}", get(get_course_by_code))
        .route("/courses/getTeachersCourse/{teacher_code}", get(get_courses_by_teacher))
        .with_state(service)
}

async fn get_course_by_id(
    State(service): State<Arc<CourseService>>,
    Path(id_course): Path<Uuid>,
) -> Result<Json<CourseResponseDto>, String> {
    let course = service
        .get_course_by_id(id_course)
        .await
        .map_err(|error| error)?;

    Ok(Json(course.into()))
}

async fn get_course_by_code(
    State(service): State<Arc<CourseService>>,
    Path(course_code): Path<Uuid>,
) -> Result<Json<CourseResponseDto>, String> {
    let course = service
        .get_course_by_id(course_code)
        .await
        .map_err(|error| error)?;

    Ok(Json(course.into()))
}

async fn get_courses_by_teacher(
    State(service): State<Arc<CourseService>>,
    Path(id_user): Path<Uuid>,
) -> Result<Json<CourseResponseDto>, String> {
    let course = service
        .get_course_by_id(id_user)
        .await
        .map_err(|error| error)?;

    Ok(Json(course.into()))
}

