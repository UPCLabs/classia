use super::dto::{ActivityCreateDto, ActivityResponseDto, ActivityUpdateDto};
use crate::{auth::AuthenticatedUser, error::AppError, state::ClassiaState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct ActivityListResponseDto {
    items: Vec<ActivityResponseDto>,
}

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/course/{course_id}", get(list_activities_by_course))
        .route("/{activity_id}", get(get_activity).patch(update_activity))
        .route("/{activity_id}/publish", post(publish_activity))
}

pub(crate) async fn create_activity(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(course_id): Path<Uuid>,
    Json(body): Json<ActivityCreateDto>,
) -> Result<(axum::http::StatusCode, Json<ActivityResponseDto>), AppError> {
    let activity = state
        .activity_service
        .create(user.id, &user.role, course_id, body)
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(activity.into())))
}

async fn update_activity(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(activity_id): Path<Uuid>,
    Json(body): Json<ActivityUpdateDto>,
) -> Result<Json<ActivityResponseDto>, AppError> {
    Ok(Json(
        state
            .activity_service
            .update(user.id, &user.role, activity_id, body)
            .await?
            .into(),
    ))
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
