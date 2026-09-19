use axum::{Router, routing::get};


pub fn route() -> Router {
    Router::new()
        .route("/users/:id_user", get(get_user_by_id))
}