pub(crate) mod api;
mod dto;
mod error;
mod models;
mod repository;
mod service;

#[cfg(test)]
mod tests;

use crate::courses::CourseService;
use sqlx::PgPool;
use std::sync::Arc;

pub(crate) use api::route;
pub(crate) use service::ActivityService;

pub(crate) fn build_service(pool: PgPool, course_service: Arc<CourseService>) -> ActivityService {
    ActivityService::new(repository::ActivityRepository::new(pool), course_service)
}
