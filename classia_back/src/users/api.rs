use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{patch, post},
};
use serde_json::{Value, json};

use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    state::ClassiaState,
    users::dto::{ChangePasswordDto, UserCreateDto},
};

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route("/create", post(create_user))
        .route("/change-password", patch(change_password))
}

async fn create_user(
    State(service): State<ClassiaState>,
    Json(body): Json<UserCreateDto>,
) -> Result<Json<Value>, AppError> {
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
