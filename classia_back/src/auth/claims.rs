use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::UserRole;

/// Distinguishes the full session from the short MFA step of the login
/// (SDS v2.2 §14.3). Tokens without a purpose are rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TokenPurpose {
    Access,
    MfaPending,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub purpose: TokenPurpose,
}
