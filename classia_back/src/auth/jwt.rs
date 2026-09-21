use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
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
        let issued_at = Utc::now().timestamp();

        let claims = Claims {
            sub: user_id,
            role: role,
            iat: issued_at,
            exp: issued_at + self.expiration_secs,
            iss: self.issuer.clone(),
        };

        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(|_| AuthError::InternalError("Fail to generate JWT???".to_string()))
    }

    pub fn validate(&self, token: String) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);

        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_required_spec_claims(&["sub", "iat", "exp", "iss"]);
        validation.validate_exp = true;

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|error| {
                use jsonwebtoken::errors::ErrorKind;

                match error.kind() {
                    ErrorKind::ExpiredSignature => {
                        AuthError::InternalError("Token expired".to_string())
                    }
                    _ => AuthError::InternalError("Token invalid".to_string()),
                }
            })
    }
}
