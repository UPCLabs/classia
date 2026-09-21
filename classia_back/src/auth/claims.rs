use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::UserRole;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
}
