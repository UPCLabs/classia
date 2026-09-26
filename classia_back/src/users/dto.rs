use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::users::User;
use crate::users::models::UserRole;
use crate::util::deserializers::{deserialize_optional_trimmed, deserialize_trimmed};

#[derive(Debug, Deserialize, Validate)]
pub struct UserCreateDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "El nombre debe tener entre 1 y 100 caracteres"
    ))]
    #[serde(deserialize_with = "deserialize_trimmed")]
    pub name: String,

    #[validate(email(message = "Correo inválido"))]
    #[serde(deserialize_with = "deserialize_trimmed")]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 20,
        message = "La contraseña debe tener entre 8 y 20 caracteres"
    ))]
    pub password: String,

    pub role: UserRole,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordDto {
    #[validate(length(
        min = 8,
        max = 20,
        message = "La contraseña debe tener entre 8 y 20 caracteres"
    ))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "El nombre debe tener entre 1 y 100 caracteres"
    ))]
    #[serde(deserialize_with = "deserialize_optional_trimmed")]
    pub name: Option<String>,

    #[validate(email(message = "Correo inválido"))]
    #[serde(deserialize_with = "deserialize_optional_trimmed")]
    pub email: Option<String>,

    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize)]
pub struct UserResponseDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
            status: user.status,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserQueryParams {
    pub q: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<String>,
}