use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{app_state::ClassiaState, users::dto::UserResponseDto};

pub fn route() -> Router<ClassiaState> {
    Router::new().route("/{id_user}", get(get_user_by_id))
}

// Same here, we need to think, how to send templates of errores
// On a easy way, [String as error, is not good, we need for example]
// A Enum/struct that have HTTP code with custom error, or something like that
async fn get_user_by_id(
    State(service): State<ClassiaState>,
    Path(id_user): Path<Uuid>,
) -> Result<Json<UserResponseDto>, String> {
    let user = service
        .user_service
        .get_user_by_id(id_user)
        .await
        .map_err(|error| error)?;

    Ok(Json(user.into()))
}
