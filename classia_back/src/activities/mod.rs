mod api;
mod dto;
mod error;
mod models;
mod repository;
mod service;

use std::sync::Arc;

use sqlx::PgPool;

use crate::courses::CourseService;

pub use api::route;
pub use service::ActivityService;

pub fn build_service(pool: PgPool, course_service: Arc<CourseService>) -> ActivityService {
    ActivityService::new(repository::ActivityRepository::new(pool), course_service)
}
