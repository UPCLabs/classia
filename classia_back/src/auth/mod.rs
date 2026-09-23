mod api;
mod claims;
mod dto;
mod error;
mod extractor;
mod jwt;
mod service;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::{
    auth::{error::AuthError, jwt::JwtService},
    users::UserService,
};

pub(crate) use api::route;
pub(crate) use extractor::AuthenticatedUser;
pub(crate) use jwt::JwtConfig;
pub(crate) use service::AuthService;

pub(crate) fn build_service(
    config: JwtConfig,
    user_service: Arc<UserService>,
) -> Result<AuthService, AuthError> {
    let jwt_service = JwtService::new(config)?;

    Ok(AuthService::new(jwt_service, user_service))
}
