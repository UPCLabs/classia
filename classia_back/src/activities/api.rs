use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};
use uuid::Uuid;

use crate::{
    activities::dto::ActivityResponseDto, auth::AuthenticatedUser, error::AppError,
    state::ClassiaState,
};

pub fn route() -> Router<ClassiaState> {
    Router::new().route("/{activity_id}/publish", post(publish_activity))
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
