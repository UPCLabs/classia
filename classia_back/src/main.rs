use std::{env, error::Error, sync::Arc};

use axum::{Router, routing::get};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{auth::JwtConfig, config::bootstrap::ensure_super_admin, state::ClassiaState};

mod auth;
mod config;
mod courses;
mod error;
mod state;
mod users;
mod util;

async fn create_state(pool: PgPool) -> Result<ClassiaState, Box<dyn Error>> {
    let secret = env::var("JWT_SECRET")?;

    let user_service = Arc::new(users::build_service(pool.clone()));
    let course_service = Arc::new(courses::build_service(pool));

    let jwt_config = JwtConfig {
        secret: secret,
        issuer: "classia".to_string(),
        expiration: 86400,
    };
    let auth_service = Arc::new(auth::build_service(jwt_config, user_service.clone()).unwrap());

    Ok(ClassiaState::new(
        user_service,
        course_service,
        auth_service,
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let port = env::var("PORT")?;
    let address = format!("0.0.0.0:{port}");
    let database_url = env::var("DATABASE_URL")?;

    info!("Connecting to database...");
    let pool = util::database::connect(&database_url).await?;

    info!("Running database migrations...");
    sqlx::migrate!().run(&pool).await?;

    info!("Checking superadmin...");
    ensure_super_admin(&pool).await?;

    info!("Initializing state...");
    let state = create_state(pool).await?;

    let app = Router::new()
        .route("/ping", get(ping))
        .nest("/auth", auth::route())
        .nest("/users", users::route())
        .nest("/courses", courses::route())
        .with_state(state);

    let listener = TcpListener::bind(&address).await?;

    info!(port = %port, "Server ON!!");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn ping() -> &'static str {
    "Pong"
}
