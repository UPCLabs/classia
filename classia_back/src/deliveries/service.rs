use super::{
    error::DeliveryError,
    models::DeliveryRow,
    repository::DeliveryRepository,
    storage::{self, UploadedFile},
};
use crate::{
    activities::{ActivityError, ActivityService},
    audit::{
        self,
        models::{AuditAction, AuditResult, AuditTarget, NewAuditEvent},
    },
    users::UserRole,
    util::database::is_unique_violation,
};
use serde_json::json;
use sqlx::PgPool;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::error;
use uuid::Uuid;

pub struct DeliveryService {
    repository: DeliveryRepository,
    activity_service: Arc<ActivityService>,
    files_dir: PathBuf,
}
impl DeliveryService {
    pub fn new(
        repository: DeliveryRepository,
        activity_service: Arc<ActivityService>,
        files_dir: PathBuf,
    ) -> Self {
        Self {
            repository,
            activity_service,
            files_dir,
        }
    }

    pub async fn get_delivery_status(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        activity_id: Uuid,
    ) -> Result<Option<DeliveryRow>, DeliveryError> {
        if *actor_role != UserRole::Student {
            return Err(DeliveryError::Forbidden);
        }

        self.activity_service
            .get_for_user(actor_id, actor_role, activity_id)
            .await?;

        self.repository
            .find_by_activity_and_student(activity_id, actor_id)
            .await
            .map_err(DeliveryError::Database)
    }

    pub async fn list_activity_deliveries(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        activity_id: Uuid,
    ) -> Result<Vec<DeliveryRow>, DeliveryError> {
        if *actor_role != UserRole::Teacher {
            return Err(DeliveryError::Forbidden);
        }

        self.activity_service
            .get_for_user(actor_id, actor_role, activity_id)
            .await?;

        self.repository
            .list_by_activity(activity_id)
            .await
            .map_err(DeliveryError::Database)
    }

    pub async fn download_delivery_file(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        delivery_id: Uuid,
    ) -> Result<Vec<u8>, DeliveryError> {
        if *actor_role != UserRole::Teacher {
            return Err(DeliveryError::Forbidden);
        }

        let (activity_id, file_url) = self
            .repository
            .find_file_location(delivery_id)
            .await
            .map_err(DeliveryError::Database)?
            .ok_or(DeliveryError::DeliveryNotFound)?;

        self.activity_service
            .get_for_user(actor_id, actor_role, activity_id)
            .await?;

        if Path::new(&file_url).file_name() != Some(OsStr::new(&file_url)) {
            error!("Invalid file_url for delivery {delivery_id}");
            return Err(DeliveryError::FileUnavailable);
        }

        tokio::fs::read(self.files_dir.join(&file_url))
            .await
            .map_err(|e| {
                error!("Could not read the file of delivery {delivery_id}: {e}");
                DeliveryError::FileUnavailable
            })
    }
}

impl DeliveryService {
    /// UC-015. The file is written first and removed again if the
    /// transaction does not commit (SDS §9.5).
    pub async fn submit(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        activity_id: Uuid,
        file: UploadedFile,
    ) -> Result<DeliveryRow, DeliveryError> {
        let result = self
            .try_submit(actor_id, actor_role, activity_id, file)
            .await;
        if let Err(error) = &result {
            record_rejection(
                self.repository.pool(),
                actor_id,
                None,
                AuditAction::SubmitDelivery,
                json!({ "activity_id": activity_id }),
                error,
            )
            .await;
        }
        result
    }

    async fn try_submit(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        activity_id: Uuid,
        file: UploadedFile,
    ) -> Result<DeliveryRow, DeliveryError> {
        if *actor_role != UserRole::Student {
            return Err(DeliveryError::Forbidden);
        }
        self.activity_service
            .get_for_user(actor_id, actor_role, activity_id)
            .await?;
        storage::validate(&file)?;

        let file_url = storage::save(&self.files_dir, &file.bytes).await?;
        let stored = self.insert_delivery(actor_id, activity_id, &file_url).await;
        let delivery_id = match stored {
            Ok(id) => id,
            Err(error) => {
                storage::remove(&self.files_dir, &file_url).await;
                return Err(error);
            }
        };

        self.find_detail(delivery_id).await
    }

    async fn insert_delivery(
        &self,
        actor_id: Uuid,
        activity_id: Uuid,
        file_url: &str,
    ) -> Result<Uuid, DeliveryError> {
        let mut tx = self.repository.pool().begin().await?;
        if !DeliveryRepository::activity_is_open(&mut tx, activity_id).await? {
            return Err(DeliveryError::DueDatePassed);
        }
        let delivery_id = DeliveryRepository::insert(&mut tx, activity_id, actor_id, file_url)
            .await
            .map_err(|error| {
                if is_unique_violation(&error) {
                    DeliveryError::AlreadyExists
                } else {
                    DeliveryError::Database(error)
                }
            })?;
        audit::repository::record(
            &mut *tx,
            &NewAuditEvent {
                actor_user_id: Some(actor_id),
                target: AuditTarget::Delivery,
                target_id: Some(delivery_id),
                action: AuditAction::SubmitDelivery,
                result: AuditResult::Success,
                details: json!({ "activity_id": activity_id, "version": 1 }),
            },
        )
        .await?;
        tx.commit().await?;
        Ok(delivery_id)
    }

    /// UC-016. A delivery of another student is reported as not found.
    pub async fn replace(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        delivery_id: Uuid,
        expected_version: Option<i32>,
        file: UploadedFile,
    ) -> Result<DeliveryRow, DeliveryError> {
        let result = self
            .try_replace(actor_id, actor_role, delivery_id, expected_version, file)
            .await;
        if let Err(error) = &result {
            record_rejection(
                self.repository.pool(),
                actor_id,
                Some(delivery_id),
                AuditAction::ReplaceDelivery,
                json!({ "expected_version": expected_version }),
                error,
            )
            .await;
        }
        result
    }

    async fn try_replace(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        delivery_id: Uuid,
        expected_version: Option<i32>,
        file: UploadedFile,
    ) -> Result<DeliveryRow, DeliveryError> {
        if *actor_role != UserRole::Student {
            return Err(DeliveryError::Forbidden);
        }
        let (activity_id, student_id) = self
            .repository
            .find_owner(delivery_id)
            .await?
            .ok_or(DeliveryError::DeliveryNotFound)?;
        if student_id != actor_id {
            return Err(DeliveryError::DeliveryNotFound);
        }
        self.activity_service
            .get_for_user(actor_id, actor_role, activity_id)
            .await?;
        let expected_version = expected_version.ok_or_else(|| {
            DeliveryError::Validation("expected_version es obligatorio y debe ser un entero".into())
        })?;
        storage::validate(&file)?;

        let file_url = storage::save(&self.files_dir, &file.bytes).await?;
        match self
            .replace_in_transaction(actor_id, delivery_id, expected_version, &file_url)
            .await
        {
            Ok(previous_file) => storage::remove(&self.files_dir, &previous_file).await,
            Err(error) => {
                storage::remove(&self.files_dir, &file_url).await;
                return Err(error);
            }
        }

        self.find_detail(delivery_id).await
    }

    /// Returns the previous file name, to be removed only after commit.
    async fn replace_in_transaction(
        &self,
        actor_id: Uuid,
        delivery_id: Uuid,
        expected_version: i32,
        file_url: &str,
    ) -> Result<String, DeliveryError> {
        let mut tx = self.repository.pool().begin().await?;
        let target = DeliveryRepository::lock_for_replace(&mut tx, delivery_id)
            .await?
            .filter(|target| target.student_id == actor_id)
            .ok_or(DeliveryError::DeliveryNotFound)?;
        if target.version != expected_version {
            return Err(DeliveryError::VersionConflict);
        }
        if !target.open {
            return Err(DeliveryError::DueDatePassed);
        }

        // CON-04: a replaced delivery loses its grade; the audit keeps it.
        let previous_evaluation = target
            .score
            .map(|score| json!({ "score": score, "feedback": target.feedback }));
        if previous_evaluation.is_some() {
            DeliveryRepository::delete_evaluation(&mut tx, delivery_id).await?;
        }
        let version = DeliveryRepository::replace_file(&mut tx, delivery_id, file_url).await?;
        audit::repository::record(
            &mut *tx,
            &NewAuditEvent {
                actor_user_id: Some(actor_id),
                target: AuditTarget::Delivery,
                target_id: Some(delivery_id),
                action: AuditAction::ReplaceDelivery,
                result: AuditResult::Success,
                details: json!({
                    "previous_version": target.version,
                    "version": version,
                    "previous_evaluation": previous_evaluation,
                }),
            },
        )
        .await?;
        tx.commit().await?;
        Ok(target.file_url)
    }

    async fn find_detail(&self, delivery_id: Uuid) -> Result<DeliveryRow, DeliveryError> {
        self.repository
            .find_detail(delivery_id)
            .await?
            .ok_or(DeliveryError::DeliveryNotFound)
    }
}

/// `denied` and `failure` events go on the pool after the rollback
/// (SDS §10.3). Input-format errors and not-found lookups are not audited.
pub(super) async fn record_rejection(
    pool: &PgPool,
    actor_id: Uuid,
    target_id: Option<Uuid>,
    action: AuditAction,
    mut details: serde_json::Value,
    error: &DeliveryError,
) {
    let (result, reason) = match error {
        DeliveryError::Forbidden | DeliveryError::Activity(ActivityError::Forbidden) => {
            (AuditResult::Denied, "FORBIDDEN")
        }
        DeliveryError::DueDatePassed => (AuditResult::Denied, "DUE_DATE_PASSED"),
        DeliveryError::AlreadyExists => (AuditResult::Denied, "DELIVERY_ALREADY_EXISTS"),
        DeliveryError::VersionConflict => (AuditResult::Denied, "DELIVERY_VERSION_CONFLICT"),
        DeliveryError::Database(_) => (AuditResult::Failure, "DATABASE_ERROR"),
        DeliveryError::FileUnavailable => (AuditResult::Failure, "FILE_UNAVAILABLE"),
        _ => return,
    };
    details["reason"] = json!(reason);
    let target = if action == AuditAction::GradeDelivery {
        AuditTarget::Evaluation
    } else {
        AuditTarget::Delivery
    };
    let event = NewAuditEvent {
        actor_user_id: Some(actor_id),
        target,
        target_id,
        action,
        result,
        details,
    };
    if let Err(audit_error) = audit::repository::record(pool, &event).await {
        error!(
            "Could not record the {} audit event: {audit_error}",
            action.as_str()
        );
    }
}
