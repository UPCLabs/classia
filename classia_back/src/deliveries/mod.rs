pub(crate) mod api;
mod dto;
mod error;
mod evaluation;
pub(crate) mod models;
mod repository;
mod service;
mod storage;

#[cfg(test)]
mod service_tests;
#[cfg(test)]
mod tests;

use crate::activities::ActivityService;
use sqlx::PgPool;
use std::{path::PathBuf, sync::Arc};

pub(crate) use api::{get_my_delivery, list_deliveries, route, submit_delivery};
pub(crate) use evaluation::EvaluationService;
pub(crate) use service::DeliveryService;
pub(crate) use storage::MAX_REQUEST_BYTES;

const DEFAULT_DELIVERIES_DIR: &str = "/var/lib/classia/deliveries";

pub(crate) fn files_dir_from_env() -> PathBuf {
    std::env::var("DELIVERIES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_DELIVERIES_DIR))
}

pub(crate) fn build_service(
    pool: PgPool,
    activity_service: Arc<ActivityService>,
    files_dir: PathBuf,
) -> DeliveryService {
    DeliveryService::new(
        repository::DeliveryRepository::new(pool),
        activity_service,
        files_dir,
    )
}

pub(crate) fn build_evaluation_service(
    pool: PgPool,
    activity_service: Arc<ActivityService>,
) -> EvaluationService {
    EvaluationService::new(repository::DeliveryRepository::new(pool), activity_service)
}
