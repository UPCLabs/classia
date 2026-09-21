use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use uuid::Uuid;

use crate::{
    app_error::AppError,
    app_state::ClassiaState,
    users::dto::{UserCreateDto, UserResponseDto},
};

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/{id_user}", get(get_user_by_id))
        .route("/email/{email}", get(get_user_by_email))
        .route("/create", post(create_user))
}

async fn get_user_by_id(
    State(service): State<ClassiaState>,
    Path(id_user): Path<Uuid>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = service.user_service.get_user_by_id(id_user).await?;

    Ok(Json(user.into()))
}

async fn get_user_by_email(
    State(service): State<ClassiaState>,
    Path(email): Path<String>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = service.user_service.get_user_by_email(&email).await?;

    Ok(Json(user.into()))
}

async fn create_user(
    State(service): State<ClassiaState>,
    Json(body): Json<UserCreateDto>,
) -> Result<Json<UserResponseDto>, AppError> {
    let user = service.user_service.create_user(body).await?;

    Ok(Json(user.into()))
}
