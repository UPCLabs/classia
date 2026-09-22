use axum::{Json, Router, extract::State, routing::post};
use serde_json::{Value, json};

use crate::{error::AppError, state::ClassiaState, users::dto::UserCreateDto};

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
    Json(body): Json<ChangePasswordDto>,
) -> Result<StatusCode, AppError> {
    //    state
    //        .user_service
    //        .change_password(current_user.id, body)
    //        .await?;
    Ok(StatusCode::NO_CONTENT)
}
