use std::sync::Arc;

use crate::{
    auth::{error::AuthError, jwt::JwtService},
    users::UserService,
};

pub struct AuthService {
    user_service: Arc<UserService>,
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(jwt_service: JwtService, user_service: Arc<UserService>) -> Self {
        Self {
            jwt_service,
            user_service,
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(), AuthError> {
        let user = self
            .user_service
            .get_user_by_email(email)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        todo!()
    }
}
