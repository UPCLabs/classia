use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{app_error::AppError, app_state::ClassiaState, users::dto::UserResponseDto};

pub fn route() -> Router<ClassiaState> {
    Router::new().route("/{id_user}", get(get_user_by_id))
}

async fn get_user_by_id(
    State(service): State<ClassiaState>,
    Path(id_user): Path<Uuid>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = service.user_service.get_user_by_id(id_user).await?;

    Ok(Json(user.into()))
}
