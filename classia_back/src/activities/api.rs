use super::dto::{ActivityCreateDto, ActivityResponseDto, ActivityUpdateDto};
use crate::{auth::AuthenticatedUser, error::AppError, state::ClassiaState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::patch,
};
use uuid::Uuid;

pub fn route() -> Router<ClassiaState> {
    Router::new().route("/{activity_id}", patch(update_activity))
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
