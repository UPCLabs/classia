use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    state::ClassiaState,
    users::dto::{
        ChangePasswordDto, UpdateUserDto, UpdateUserStatusDto, UserCreateDto, UserListResponseDto,
        UserQueryParams, UserResponseDto,
    },
};

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/", get(list_users))
        .route("/create", post(create_user))
        .route("/change-password", patch(change_password))
        .route("/{id}", get(get_user).patch(update_user))
        .route("/{id}/status", patch(update_user_status))
}

fn require_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.role.is_admin() {
        Ok(())
    } else {
        Err(AppError::forbidden())
    }
}

async fn list_users(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
    Query(params): Query<UserQueryParams>,
) -> Result<Json<UserListResponseDto>, AppError> {
    require_admin(&user)?;

    let users = state.user_service.list_users(params).await?;

    Ok(Json(UserListResponseDto {
        items: users.into_iter().map(Into::into).collect(),
    }))
}

async fn get_user(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponseDto>, AppError> {
    require_admin(&user)?;

    let target = state.user_service.get_user_by_id(id).await?;

    Ok(Json(target.into()))
}

async fn create_user(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
    Json(body): Json<UserCreateDto>,
) -> Result<(StatusCode, Json<UserResponseDto>), AppError> {
    require_admin(&user)?;

    let created = state.user_service.create_user(body, user.role).await?;

    Ok((StatusCode::CREATED, Json(created.into())))
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
) -> Result<Json<UserResponseDto>, AppError> {
    require_admin(&user)?;

    let updated = state.user_service.update_user(id, body, user.role).await?;

    Ok(Json(updated.into()))
}

async fn update_user_status(
    State(state): State<ClassiaState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserStatusDto>,
) -> Result<Json<UserResponseDto>, AppError> {
    require_admin(&user)?;

    let updated = state
        .user_service
        .update_user_status(id, body.status, user.id, user.role)
        .await?;

    Ok(Json(updated.into()))
}
