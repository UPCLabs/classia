use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, routing::{delete, get, patch, post},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedUser, error::AppError, state::ClassiaState, users::{
        UserRole, dto::{ChangePasswordDto, UpdateUserDto, UserCreateDto, UserResponseDto},
    },
};

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/create", post(create_user))
        .route("/change-password", patch(change_password))
        .route("/update/:id", patch(update_user))
        .route("/delete/:id", delete(delete_user))
        .route("/getAll", get(list_users))
}

pub(super) fn can_create_user(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin)
}

async fn create_user(
    State(service): State<ClassiaState>,
    user: AuthenticatedUser,
    Json(body): Json<UserCreateDto>,
) -> Result<Json<Value>, AppError> {
    if !can_create_user(&user.role) {
        return Err(AppError::forbidden());
    }

    let _ = service.user_service.create_user(body).await?;

    Ok(Json(json!({
        "message": "User created"
    })))
}

async fn change_password(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
    Json(request): Json<ChangePasswordDto>,
) -> Result<StatusCode, AppError> {
    state.user_service.change_password(user.id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_user(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserDto>,
) -> Result<Json<Value>, AppError> {
    if !can_create_user(&user.role) {
        return Err(AppError::forbidden());
    }

    let _ = state.user_service.update_user(id, body).await?;

    Ok(Json(json!({
        "message": "User updated"
    })))
}

async fn delete_user(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if !can_create_user(&user.role) {
        return Err(AppError::forbidden());
    }

    state.user_service.delete_user(id).await?;

    Ok(Json(json!({
        "message": "User deleted"
    })))
}

async fn list_users(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
) -> Result<Json<Value>, AppError> {

    if !can_create_user(&user.role) {
        return Err(AppError::forbidden());
    }

    let users = state.user_service.list_users().await?;

    let users_dto: Vec<UserResponseDto> = users.into_iter().map(|u| u.into()).collect();

    Ok(Json(json!(users_dto)))
}