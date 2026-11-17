mod api;
mod claims;
mod dto;
mod error;
mod extractor;
mod jwt;
mod mfa;
mod mfa_repository;
mod service;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    auth::{error::AuthError, jwt::JwtService, mfa::MfaCipher},
    users::UserService,
};

pub(crate) use api::route;
pub(crate) use extractor::AuthenticatedUser;
pub(crate) use jwt::JwtConfig;
pub(crate) use service::AuthService;

pub(crate) fn build_service(
    config: JwtConfig,
    user_service: Arc<UserService>,
    pool: PgPool,
    mfa_key: Option<&str>,
) -> Result<AuthService, AuthError> {
    let jwt_service = JwtService::new(config)?;
    let cipher = mfa_key.map(MfaCipher::from_hex).transpose()?;

    Ok(AuthService::new(jwt_service, user_service, pool, cipher))
}
