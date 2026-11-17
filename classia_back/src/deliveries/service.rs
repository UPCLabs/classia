use super::{error::DeliveryError, models::DeliveryRow, repository::DeliveryRepository};
use crate::{activities::ActivityService, users::UserRole};
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
