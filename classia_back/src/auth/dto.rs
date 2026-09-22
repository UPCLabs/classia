use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::{User, UserRole};

#[derive(Debug, Serialize)]
pub struct IdentityResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

impl From<User> for IdentityResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            role: value.role,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
