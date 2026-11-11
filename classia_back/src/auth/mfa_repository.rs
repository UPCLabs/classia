//! Persistence of the per-user MFA state (SDS v2.2 §14.5).

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MfaStatus {
    Disabled,
    Pending,
    Active,
}

impl MfaStatus {
    fn parse(value: &str) -> Self {
        match value {
            "pending" => Self::Pending,
            "active" => Self::Active,
            _ => Self::Disabled,
        }
    }
}

#[derive(Debug)]
pub struct MfaState {
    pub email: String,
    pub status: MfaStatus,
    pub ciphertext: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub last_step: Option<i64>,
    pub failed_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

type StateRow = (
    String,
    String,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<i64>,
    i32,
    Option<DateTime<Utc>>,
);

fn into_state(row: StateRow) -> MfaState {
    let (email, status, ciphertext, nonce, last_step, failed_attempts, locked_until) = row;
    MfaState {
        email,
        status: MfaStatus::parse(&status),
        ciphertext,
        nonce,
        last_step,
        failed_attempts,
        locked_until,
    }
}

pub async fn status(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<MfaStatus>> {
    let status: Option<String> = sqlx::query_scalar("SELECT mfa_status FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    Ok(status.as_deref().map(MfaStatus::parse))
}

/// Locks the row so concurrent challenges with the same code are serialized.
pub async fn lock_active_user(
    connection: &mut PgConnection,
    user_id: Uuid,
) -> sqlx::Result<Option<MfaState>> {
    let row: Option<StateRow> = sqlx::query_as(
        "SELECT email, mfa_status, mfa_secret_ciphertext, mfa_nonce, mfa_last_step,
                mfa_failed_attempts, mfa_locked_until
         FROM users
         WHERE id = $1 AND status = 'active'
         FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(connection)
    .await?;

    Ok(row.map(into_state))
}

/// Replaces any previous pending secret; the factor stays inactive until the
/// first valid code.
pub async fn start_setup(
    connection: &mut PgConnection,
    user_id: Uuid,
    ciphertext: &[u8],
    nonce: &[u8],
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE users
         SET mfa_status = 'pending', mfa_secret_ciphertext = $2, mfa_nonce = $3,
             mfa_last_step = NULL, mfa_failed_attempts = 0, mfa_locked_until = NULL,
             mfa_enabled_at = NULL, updated_at = now()
         WHERE id = $1",
    )
    .bind(user_id)
    .bind(ciphertext)
    .bind(nonce)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn record_success(
    connection: &mut PgConnection,
    user_id: Uuid,
    step: i64,
    activate: bool,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE users
         SET mfa_last_step = $2, mfa_failed_attempts = 0, mfa_locked_until = NULL,
             mfa_status = CASE WHEN $3 THEN 'active' ELSE mfa_status END,
             mfa_enabled_at = CASE WHEN $3 THEN now() ELSE mfa_enabled_at END,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(user_id)
    .bind(step)
    .bind(activate)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn record_failure(
    connection: &mut PgConnection,
    user_id: Uuid,
    failed_attempts: i32,
    locked_until: Option<DateTime<Utc>>,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE users
         SET mfa_failed_attempts = $2, mfa_locked_until = $3
         WHERE id = $1",
    )
    .bind(user_id)
    .bind(failed_attempts)
    .bind(locked_until)
    .execute(connection)
    .await?;

    Ok(())
}
