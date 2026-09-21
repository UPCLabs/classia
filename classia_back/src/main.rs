use std::{env, error::Error, sync::Arc};

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::state::ClassiaState;

mod auth;
mod courses;
mod error;
mod state;
mod users;
mod util;

async fn create_state() -> Result<ClassiaState, Box<dyn Error>> {
    let database_url = env::var("DATABASE_URL")?;
    let pool = util::database::connect(&database_url).await?;
    let user_service = Arc::new(users::build_service(pool.clone()));
    let course_service = Arc::new(courses::build_service(pool));

    Ok(ClassiaState::new(user_service, course_service))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let port = env::var("PORT").expect("env var PORT is not set");
    let address = format!("0.0.0.0:{}", port);

    info!("State initing");
    let state = match create_state().await {
        Ok(state) => state,
        Err(error) => {
            error!("{}", error);
            return;
        }
    };

    let app = Router::new()
        .route("/ping", get(ping))
        .nest("/users", users::route())
        .nest("/courses", courses::route())
        .with_state(state);

    let listener = TcpListener::bind(&address).await.unwrap();

    info!(port = port, "Server ON!!");

    axum::serve(listener, app).await.unwrap();
}

async fn ping() -> &'static str {
    "Pong"
}
