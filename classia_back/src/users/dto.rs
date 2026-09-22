use serde::Deserialize;
use validator::Validate;

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
