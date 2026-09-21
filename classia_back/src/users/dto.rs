use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::users::models::{User, UserRole};

#[derive(Debug, Serialize)]
pub struct UserResponseDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponseDto {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            role: value.role,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserCreateDto {
    #[validate(length(min = 1, max = 100, message = "El nombre debe tener entre 1 y 100 caracteres"))]
    pub name: String,

    #[validate(email(message = "Correo inválido"))]
    pub email: String,

    #[validate(length(min = 8, max = 20, message = "La contraseña debe tener entre 8 y 20 caracteres"))]
    pub password: String,

    pub role: UserRole,
}