use jsonwebtoken::{DecodingKey, EncodingKey};
use uuid::Uuid;

use crate::{
    auth::{claims::Claims, error::AuthError},
    users::UserRole,
};

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    expiration_secs: i64,
}

impl JwtService {
    pub fn new(secret: &str, issuer: String, expiration: i64) -> Result<Self, AuthError> {
        if secret.len() < 32 {
            return Err(AuthError::InternalError(String::from(
                "Secret is too short",
            )));
        }

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer,
            expiration_secs: expiration,
        })
    }

    pub fn generate(&self, user_id: Uuid, role: UserRole) -> Result<String, AuthError> {
        todo!()
    }

    pub fn validate(&self, token: String) -> Result<Claims, AuthError> {
        todo!()
    }
}
