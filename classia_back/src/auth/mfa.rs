//! TOTP and secret protection for UC-005 (SDS v2.2 §14.3–14.6).

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
};
use totp_rs::{Secret, Totp};
use uuid::Uuid;

use crate::auth::error::AuthError;

pub const ISSUER: &str = "Classia";
pub const MAX_FAILED_ATTEMPTS: i32 = 5;
pub const LOCK_MINUTES: i64 = 5;

/// AES-256-GCM key loaded from `MFA_ENCRYPTION_KEY` (64 hex characters).
/// Kept separate from the JWT secret; MFA never falls back to plain text.
#[derive(Clone)]
pub struct MfaCipher {
    cipher: Aes256Gcm,
}

pub struct EncryptedSecret {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub struct NewTotp {
    pub secret_base32: String,
    pub url: String,
    pub raw_secret: Vec<u8>,
}

impl MfaCipher {
    pub fn from_hex(hex_key: &str) -> Result<Self, AuthError> {
        let bytes = decode_hex(hex_key.trim())
            .filter(|bytes| bytes.len() == 32)
            .ok_or_else(|| {
                AuthError::InternalError(
                    "MFA_ENCRYPTION_KEY must be 32 bytes in hexadecimal".to_string(),
                )
            })?;

        let cipher = Aes256Gcm::new_from_slice(&bytes)
            .map_err(|_| AuthError::InternalError("Invalid MFA_ENCRYPTION_KEY".to_string()))?;

        Ok(Self { cipher })
    }

    /// The user id is bound as associated data so a ciphertext copied to
    /// another row does not decrypt.
    pub fn encrypt(&self, user_id: Uuid, secret: &[u8]) -> Result<EncryptedSecret, AuthError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: secret,
                    aad: user_id.as_bytes(),
                },
            )
            .map_err(|_| AuthError::InternalError("MFA secret encryption failed".to_string()))?;

        Ok(EncryptedSecret {
            ciphertext,
            nonce: nonce.to_vec(),
        })
    }

    pub fn decrypt(
        &self,
        user_id: Uuid,
        ciphertext: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, AuthError> {
        let nonce: [u8; 12] = nonce
            .try_into()
            .map_err(|_| AuthError::InternalError("Invalid MFA nonce".to_string()))?;

        self.cipher
            .decrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: ciphertext,
                    aad: user_id.as_bytes(),
                },
            )
            .map_err(|_| AuthError::InternalError("MFA secret decryption failed".to_string()))
    }
}

pub fn generate_totp(email: &str) -> Result<NewTotp, AuthError> {
    let totp = totp_rs::Builder::new()
        .with_issuer(Some(ISSUER))
        .with_account_name(email)
        .build()
        .map_err(|error| AuthError::InternalError(error.to_string()))?;

    let url = totp
        .to_url()
        .map_err(|error| AuthError::InternalError(error.to_string()))?;

    Ok(NewTotp {
        secret_base32: totp.secret().to_base32(),
        url,
        raw_secret: totp.secret().to_vec(),
    })
}

fn totp_from_secret(secret: &[u8]) -> Result<Totp, AuthError> {
    totp_rs::Builder::new()
        .with_secret(Secret::from(secret.to_vec()))
        .build()
        .map_err(|error| AuthError::InternalError(error.to_string()))
}

/// Returns the matched time step if `code` is valid at `unix_time`
/// (RFC 6238: SHA-1, 6 digits, 30 s, ±1 step).
pub fn verify_code(secret: &[u8], code: &str, unix_time: u64) -> Result<Option<u64>, AuthError> {
    if !is_six_digits(code) {
        return Ok(None);
    }

    Ok(totp_from_secret(secret)?.check(code, unix_time))
}

pub fn is_six_digits(code: &str) -> bool {
    code.len() == 6 && code.bytes().all(|byte| byte.is_ascii_digit())
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }

    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(value.get(index..index + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
pub fn code_at(secret: &[u8], unix_time: u64) -> String {
    totp_from_secret(secret)
        .expect("valid test secret")
        .generate(unix_time)
        .to_string()
}
