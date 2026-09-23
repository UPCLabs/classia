mod api;
mod dto;
mod error;
mod models;
mod repository;
mod service;

#[cfg(test)]
mod tests;

use sqlx::PgPool;

pub(crate) use api::route;
pub(crate) use service::CourseService;

pub(crate) fn build_service(pool: PgPool) -> CourseService {
    let repository = repository::CourseRepository::new(pool);
    CourseService::new(repository)
}
