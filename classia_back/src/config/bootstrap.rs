use std::error::Error;

use sqlx::PgPool;
use tracing::info;

use crate::util::password::hash_password;

pub(crate) struct BootstrapAdminConfig {
    pub name: String,
    pub email: String,
    pub password: String,
}

pub(crate) async fn ensure_super_admin(
    pool: &PgPool,
    config: BootstrapAdminConfig,
) -> Result<(), Box<dyn Error>> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
          SELECT EXISTS (
              SELECT 1
              FROM users
              WHERE role = 'super_admin'
          )
          "#,
    )
    .fetch_one(pool)
    .await?;

    if exists {
        info!("Superadmin already exists");
        return Ok(());
    }

    let password_hash = hash_password(&config.password)?;

    sqlx::query(
        r#"
          INSERT INTO users (
              name,
              email,
              password,
              role,
              status,
              token
          )
          VALUES (
              $1,
              $2,
              $3,
              'super_admin',
              'active',
              ''
          )
          "#,
    )
    .bind(config.name.trim())
    .bind(config.email.trim().to_lowercase())
    .bind(password_hash)
    .execute(pool)
    .await?;

    tracing::info!(
        email = %config.email,
        "bootstrap super administrator created"
    );

    Ok(())
}
