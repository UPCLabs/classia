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
pub(crate) use service::{CourseService, EnrollmentService};

pub(crate) fn build_service(
    pool: PgPool,
    user_service: std::sync::Arc<crate::users::UserService>,
) -> CourseService {
    let repository = repository::CourseRepository::new(pool);
    CourseService::new(repository, user_service)
}

pub(crate) fn build_enrollment_service(
    pool: PgPool,
    user_service: std::sync::Arc<crate::users::UserService>,
    course_service: std::sync::Arc<CourseService>,
) -> EnrollmentService {
    let repository = repository::CourseRepository::new(pool);
    EnrollmentService::new(repository, user_service, course_service)
}
