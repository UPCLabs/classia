use sqlx::PgExecutor;

use super::models::NewAuditEvent;

/// Takes any executor so `success` events join the caller's transaction, while
/// `denied` and `failure` events can be written on the pool after a rollback
/// (SDS §10.3).
pub async fn record<'e>(executor: impl PgExecutor<'e>, event: &NewAuditEvent) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO audit_events
            (actor_user_id, target_type, target_id, action, result, details)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(event.actor_user_id)
    .bind(event.target.as_str())
    .bind(event.target_id)
    .bind(event.action.as_str())
    .bind(event.result)
    .bind(&event.details)
    .execute(executor)
    .await?;

    Ok(())
}
