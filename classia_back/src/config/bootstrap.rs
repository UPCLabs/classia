use std::{env, error::Error, io};

use sqlx::PgPool;
use tracing::info;

use crate::util::password::hash_password;

struct BootstrapAdminConfig {
    name: String,
    email: String,
    password: String,
}

impl BootstrapAdminConfig {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name: required_env("BOOTSTRAP_ADMIN_NAME")?,
            email: required_env("BOOTSTRAP_ADMIN_EMAIL")?,
            password: required_env("BOOTSTRAP_ADMIN_PASSWORD")?,
        })
    }
}

fn required_env(name: &'static str) -> Result<String, Box<dyn Error>> {
    let value = env::var(name).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} must be set when no super administrator exists"),
        )
    })?;

    if value.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} must not be empty when no super administrator exists"),
        )
        .into());
    }

    Ok(value)
}

pub(crate) async fn ensure_super_admin(pool: &PgPool) -> Result<(), Box<dyn Error>> {
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

    let config = BootstrapAdminConfig::from_env()?;
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
