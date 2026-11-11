use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    models::{AuditAction, AuditResult, AuditTarget, NewAuditEvent},
    repository::record,
};

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO users (name, email, password, role, status, token)
         VALUES ('Teacher', 'teacher@example.com', 'hash', 'teacher', 'active', '')
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("test user is inserted")
}

fn grade_event(actor_user_id: Uuid, result: AuditResult) -> NewAuditEvent {
    NewAuditEvent {
        actor_user_id: Some(actor_user_id),
        target: AuditTarget::Evaluation,
        target_id: Some(Uuid::now_v7()),
        action: AuditAction::GradeDelivery,
        result,
        details: json!({ "previous": null, "new": { "score": 4.5 } }),
    }
}

async fn count_events(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events")
        .fetch_one(pool)
        .await
        .expect("audit events are counted")
}

#[test]
fn audit_identifiers_match_the_sds_vocabulary() {
    assert_eq!(AuditTarget::Delivery.as_str(), "delivery");
    assert_eq!(AuditTarget::Evaluation.as_str(), "evaluation");
    assert_eq!(AuditTarget::User.as_str(), "user");
    assert_eq!(AuditAction::SubmitDelivery.as_str(), "submit_delivery");
    assert_eq!(AuditAction::ReplaceDelivery.as_str(), "replace_delivery");
    assert_eq!(AuditAction::GradeDelivery.as_str(), "grade_delivery");
    assert_eq!(AuditAction::MfaSetupStarted.as_str(), "mfa.setup_started");
    assert_eq!(AuditAction::MfaActivated.as_str(), "mfa.activated");
    assert_eq!(AuditAction::MfaChallenge.as_str(), "mfa.challenge");
}

#[sqlx::test(migrations = "./migrations")]
async fn records_an_event_with_all_fields(pool: PgPool) {
    let actor_id = insert_user(&pool).await;
    let event = grade_event(actor_id, AuditResult::Success);

    record(&pool, &event).await.expect("event is recorded");

    let (actor, target_type, target_id, action, result, details, correlation): (
        Option<Uuid>,
        String,
        Option<Uuid>,
        String,
        AuditResult,
        Value,
        Option<Uuid>,
    ) = sqlx::query_as(
        "SELECT actor_user_id, target_type, target_id, action, result, details,
                correlation_id
         FROM audit_events",
    )
    .fetch_one(&pool)
    .await
    .expect("event is read back");

    assert_eq!(actor, Some(actor_id));
    assert_eq!(target_type, "evaluation");
    assert_eq!(target_id, event.target_id);
    assert_eq!(action, "grade_delivery");
    assert_eq!(result, AuditResult::Success);
    assert_eq!(details, event.details);
    assert_eq!(correlation, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn success_event_rolls_back_with_its_transaction(pool: PgPool) {
    let actor_id = insert_user(&pool).await;
    let mut transaction = pool.begin().await.expect("transaction starts");

    record(
        &mut *transaction,
        &grade_event(actor_id, AuditResult::Success),
    )
    .await
    .expect("event is recorded inside the transaction");
    transaction
        .rollback()
        .await
        .expect("transaction rolls back");

    assert_eq!(count_events(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn denied_event_persists_independently_after_a_rollback(pool: PgPool) {
    let actor_id = insert_user(&pool).await;
    let transaction = pool.begin().await.expect("transaction starts");
    transaction
        .rollback()
        .await
        .expect("transaction rolls back");

    record(&pool, &grade_event(actor_id, AuditResult::Denied))
        .await
        .expect("denied event is recorded on the pool");

    assert_eq!(count_events(&pool).await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_the_actor_keeps_the_event(pool: PgPool) {
    let actor_id = insert_user(&pool).await;
    record(&pool, &grade_event(actor_id, AuditResult::Failure))
        .await
        .expect("event is recorded");

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(actor_id)
        .execute(&pool)
        .await
        .expect("actor is deleted");

    let actor: Option<Uuid> = sqlx::query_scalar("SELECT actor_user_id FROM audit_events")
        .fetch_one(&pool)
        .await
        .expect("event remains");
    assert_eq!(actor, None);
}
