use std::{error::Error, io, sync::Arc};

use axum::{Router, routing::get};
use sqlx::PgPool;

use crate::{auth::JwtConfig, state::ClassiaState};

mod auth;
mod config;
mod courses;
mod error;
mod state;
mod users;
mod util;

pub struct JwtSettings {
    pub secret: String,
    pub issuer: String,
    pub expiration: i64,
}

pub async fn connect_database(database_url: &str) -> Result<PgPool, Box<dyn Error>> {
    util::database::connect(database_url).await
}

pub async fn initialize_database(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::migrate!().run(pool).await?;
    config::bootstrap::ensure_super_admin(pool).await
}

pub fn build_router(pool: PgPool, jwt: JwtSettings) -> Result<Router, Box<dyn Error>> {
    let user_service = Arc::new(users::build_service(pool.clone()));
    let course_service = Arc::new(courses::build_service(pool));
    let auth_service = Arc::new(
        auth::build_service(
            JwtConfig {
                secret: jwt.secret,
                issuer: jwt.issuer,
                expiration: jwt.expiration,
            },
            user_service.clone(),
        )
        .map_err(|error| io::Error::other(format!("{error:?}")))?,
    );

    let state = ClassiaState::new(user_service, course_service, auth_service);

    Ok(Router::new()
        .route("/ping", get(ping))
        .nest("/auth", auth::route())
        .nest("/users", users::route())
        .nest("/courses", courses::route())
        .with_state(state))
}

async fn ping() -> &'static str {
    "Pong"
}
