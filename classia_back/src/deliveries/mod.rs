pub(crate) mod api;
mod dto;
mod error;
pub(crate) mod models;
mod repository;
mod service;

#[cfg(test)]
mod service_tests;
#[cfg(test)]
mod tests;

use crate::activities::ActivityService;
use sqlx::PgPool;
use std::{path::PathBuf, sync::Arc};

pub(crate) use api::{get_my_delivery, list_deliveries, route};
pub(crate) use service::DeliveryService;

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
