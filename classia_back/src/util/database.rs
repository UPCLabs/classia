use std::{error::Error, time::Duration};

use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::time::timeout;

pub async fn connect(database_url: &str) -> Result<PgPool, Box<dyn Error>> {
    let pool = timeout(
        Duration::from_secs(5),
        PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url),
    )
    .await??;

    Ok(pool)
}

pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db_error) if db_error.is_unique_violation())
}
