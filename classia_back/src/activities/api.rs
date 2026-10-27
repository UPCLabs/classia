use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    activities::dto::ActivityResponseDto, auth::AuthenticatedUser, error::AppError,
    state::ClassiaState,
};

#[derive(Debug, Serialize)]
struct ActivityListResponseDto {
    items: Vec<ActivityResponseDto>,
}

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/course/{course_id}", get(list_activities_by_course))
        .route("/{activity_id}", get(get_activity))
        .route("/{activity_id}/publish", post(publish_activity))
}

async fn list_activities_by_course(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
) -> Result<Json<ActivityListResponseDto>, AppError> {
    let activities = state
        .activity_service
        .list_for_user(user.id, &user.role, course_id)
        .await?;

    Ok(Json(ActivityListResponseDto {
        items: activities.into_iter().map(Into::into).collect(),
    }))
}

async fn get_activity(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<ActivityResponseDto>, AppError> {
    let activity = state
        .activity_service
        .get_for_user(user.id, &user.role, activity_id)
        .await?;

    Ok(Json(activity.into()))
}

async fn publish_activity(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<ActivityResponseDto>, AppError> {
    let activity = state
        .activity_service
        .publish(user.id, &user.role, activity_id)
        .await?;

    Ok(Json(activity.into()))
}
