mod api;
mod dto;
mod error;
mod models;
mod repository;
mod service;

use sqlx::PgPool;

pub(crate) use api::route;
pub(crate) use service::UserService;
pub(crate) fn build_service(pool: PgPool) -> UserService {
    let repository = repository::UserRepository::new(pool);
    UserService::new(repository)
}
