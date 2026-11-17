//! UC-018: grade or correct the grade of a delivery (DEV-14, DEC-02).

use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use super::{
    dto::EvaluationInputDto, error::DeliveryError, models::DeliveryRow,
    repository::DeliveryRepository, service::record_rejection,
};
use crate::{
    activities::ActivityService,
    audit::{
        self,
        models::{AuditAction, AuditResult, AuditTarget, NewAuditEvent},
    },
    users::UserRole,
};

const MAX_FEEDBACK_CHARS: usize = 5000;

pub struct EvaluationService {
    repository: DeliveryRepository,
    activity_service: Arc<ActivityService>,
}

impl EvaluationService {
    pub fn new(repository: DeliveryRepository, activity_service: Arc<ActivityService>) -> Self {
        Self {
            repository,
            activity_service,
        }
    }

    pub async fn grade(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        delivery_id: Uuid,
        input: EvaluationInputDto,
    ) -> Result<DeliveryRow, DeliveryError> {
        let expected_version = input.expected_version;
        let result = self
            .try_grade(actor_id, actor_role, delivery_id, input)
            .await;
        if let Err(error) = &result {
            record_rejection(
                self.repository.pool(),
                actor_id,
                Some(delivery_id),
                AuditAction::GradeDelivery,
                json!({ "expected_version": expected_version }),
                error,
            )
            .await;
        }
        result
    }

    async fn try_grade(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        delivery_id: Uuid,
        input: EvaluationInputDto,
    ) -> Result<DeliveryRow, DeliveryError> {
        if *actor_role != UserRole::Teacher {
            return Err(DeliveryError::Forbidden);
        }
        let (activity_id, _) = self
            .repository
            .find_owner(delivery_id)
            .await?
            .ok_or(DeliveryError::DeliveryNotFound)?;
        self.activity_service
            .get_for_user(actor_id, actor_role, activity_id)
            .await?;

        let score = validate_score(input.score)?;
        let feedback = normalize_feedback(input.feedback)?;

        let mut tx = self.repository.pool().begin().await?;
        let version = DeliveryRepository::mark_graded(&mut tx, delivery_id, input.expected_version)
            .await?
            .ok_or(DeliveryError::VersionConflict)?;
        let previous = DeliveryRepository::lock_evaluation(&mut tx, delivery_id).await?;
        DeliveryRepository::upsert_evaluation(
            &mut tx,
            delivery_id,
            actor_id,
            score,
            feedback.as_deref(),
        )
        .await?;
        audit::repository::record(
            &mut *tx,
            &NewAuditEvent {
                actor_user_id: Some(actor_id),
                target: AuditTarget::Evaluation,
                target_id: Some(delivery_id),
                action: AuditAction::GradeDelivery,
                result: AuditResult::Success,
                details: json!({
                    "version": version,
                    "previous": previous.map(|p| json!({ "score": p.score, "feedback": p.feedback })),
                    "new": { "score": score, "feedback": feedback },
                }),
            },
        )
        .await?;
        tx.commit().await?;

        self.repository
            .find_detail(delivery_id)
            .await?
            .ok_or(DeliveryError::DeliveryNotFound)
    }
}

/// 0.0–5.0 with at most one decimal (UC-018, alternate path 2A).
pub(super) fn validate_score(score: f64) -> Result<f64, DeliveryError> {
    let tenths = score * 10.0;
    let has_one_decimal = (tenths - tenths.round()).abs() < 1e-9;
    if !score.is_finite() || !(0.0..=5.0).contains(&score) || !has_one_decimal {
        return Err(DeliveryError::Validation(
            "La nota debe estar entre 0,0 y 5,0 con máximo un decimal".into(),
        ));
    }
    Ok(tenths.round() / 10.0)
}

pub(super) fn normalize_feedback(
    feedback: Option<String>,
) -> Result<Option<String>, DeliveryError> {
    let feedback = feedback
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if feedback
        .as_ref()
        .is_some_and(|value| value.chars().count() > MAX_FEEDBACK_CHARS)
    {
        return Err(DeliveryError::Validation(
            "La retroalimentación admite máximo 5000 caracteres".into(),
        ));
    }
    Ok(feedback)
}
