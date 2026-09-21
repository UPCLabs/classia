use axum::{
    Json, Router, extract::{Path, State}, routing::{get, patch, post},
};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::ClassiaState,
    users::dto::{ChangePasswordDto, UserCreateDto, UserResponseDto},
};

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/{id_user}", get(get_user_by_id))
        .route("/create", post(create_user))
        .route("/change-password", patch(change_password))
}

async fn get_user_by_id(
    State(service): State<ClassiaState>,
    Path(id_user): Path<Uuid>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = service.user_service.get_user_by_id(id_user).await?;

    Ok(Json(user.into()))
}

async fn create_user(
    State(service): State<ClassiaState>,
    Json(body): Json<UserCreateDto>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = service.user_service.create_user(body).await?;

    Ok(Json(user.into()))
}

async fn change_password(
    State(state): State<ClassiaState>,        
    Json(body): Json<ChangePasswordDto>, 
) -> Result<StatusCode, AppError> {
//    state
//        .user_service
//        .change_password(current_user.id, body)
//        .await?;
    Ok(StatusCode::NO_CONTENT)
}
