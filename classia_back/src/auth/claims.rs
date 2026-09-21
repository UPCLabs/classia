use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::UserRole;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
}
