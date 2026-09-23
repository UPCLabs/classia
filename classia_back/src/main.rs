use std::{env, error::Error};

use classia_back::{JwtSettings, build_router, connect_database, initialize_database};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

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
    let pool = connect_database(&database_url).await?;

    info!("Running database migrations...");
    initialize_database(&pool).await?;

    info!("Initializing state...");
    let app = build_router(
        pool,
        JwtSettings {
            secret: env::var("JWT_SECRET")?,
            issuer: "classia".to_string(),
            expiration: 86400,
        },
    )?;

    let listener = TcpListener::bind(&address).await?;

    info!(port = %port, "Server ON!!");

    axum::serve(listener, app).await?;

    Ok(())
}
