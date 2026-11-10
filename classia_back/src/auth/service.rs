use std::sync::Arc;

use uuid::Uuid;

use crate::{
    auth::{
        claims::Claims,
        dto::LoginRequest,
        error::AuthError,
        jwt::JwtService,
        mfa::{self, TotpSetup},
    },
    users::{User, UserService},
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

    async fn get_user(&self, user_id: Uuid) -> Result<User, AuthError> {
        self.user_service
            .get_user_by_id(user_id)
            .await
            .map_err(|_| AuthError::InternalError(String::from("No se encontro el usuario")))
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

    pub async fn setup_mfa(&self, user_id: Uuid) -> Result<TotpSetup, AuthError> {
        let user = self.get_user(user_id).await?;

        if user.token.is_some() {
            return Err(AuthError::InvalidParamater(String::from(
                "Mfa is already set",
            )));
        }

        let totp = mfa::generate_otp(&user.email)?;

        self.user_service
            .update_mfa(user_id, &totp.totp)
            .await
            .map_err(|_| AuthError::InternalError("Can't update".to_string()))?;

        Ok(totp)
    }

    pub async fn challenge_mfa(&self, user_id: Uuid, challenge: &str) -> Result<bool, AuthError> {
        let user = self.get_user(user_id).await?;

        let Some(totp) = user.token else {
            return Err(AuthError::InvalidParamater(String::from("MFA is not set")));
        };

        Ok(mfa::validate_otp(&totp, challenge))
    }
}
