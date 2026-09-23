use std::sync::Arc;

use crate::{
    auth::{claims::Claims, dto::LoginRequest, error::AuthError, jwt::JwtService},
    users::UserService,
    util::password::validate_password,
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

    pub async fn login(&self, login: LoginRequest) -> Result<String, AuthError> {
        let user = self
            .user_service
            .get_user_by_email(&login.email)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        if !validate_password(&login.password, &user.password)
            .map_err(|e| AuthError::InternalError(e.to_string()))?
        {
            return Err(AuthError::InvalidCredentials);
        }

        if user.status != "active" {
            return Err(AuthError::InactiveUser);
        }

        self.jwt_service.generate(&user)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        self.jwt_service.validate(token)
    }
}
