use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::{
    auth::{claims::Claims, error::AuthError},
    users::User,
};

pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub expiration: i64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    expiration_secs: i64,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Result<Self, AuthError> {
        if config.secret.len() < 32 {
            return Err(AuthError::InternalError(String::from(
                "Secret is too short",
            )));
        }

        Ok(Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            issuer: config.issuer,
            expiration_secs: config.expiration,
        })
    }

    pub fn generate(&self, user: &User) -> Result<String, AuthError> {
        let issued_at = Utc::now().timestamp();

        let claims = Claims {
            sub: user.id,
            role: user.role.clone(),
            iat: issued_at,
            exp: issued_at + self.expiration_secs,
            iss: self.issuer.clone(),
        };

        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(|_| AuthError::InternalError("Fail to generate JWT???".to_string()))
    }

    pub fn validate(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);

        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_required_spec_claims(&["sub", "iat", "exp", "iss"]);
        validation.validate_exp = true;

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|error| {
                use jsonwebtoken::errors::ErrorKind;

                match error.kind() {
                    ErrorKind::ExpiredSignature => AuthError::JwtError("Token expired".to_string()),
                    _ => AuthError::JwtError("Token invalid".to_string()),
                }
            })
    }
}
