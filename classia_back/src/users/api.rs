use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::users::{dto::UserResponseDto, repository::UserRepository, service::UserService};

pub fn route(pool: PgPool) -> Router {
    let repository = UserRepository::new(pool);
    let service = Arc::new(UserService::new(repository));

    Router::new()
        .route("/users/{id_user}", get(get_user_by_id))
        .with_state(service)
}

// Same here, we need to think, how to send templates of errores
// On a easy way, [String as error, is not good, we need for example]
// A Enum/struct that have HTTP code with custom error, or something like that
async fn get_user_by_id(
    State(service): State<Arc<UserService>>,
    Path(id_user): Path<Uuid>,
) -> Result<Json<UserResponseDto>, String> {
    let user = service
        .get_user_by_id(id_user)
        .await
        .map_err(|error| error)?;

    Ok(Json(user.into()))
}
