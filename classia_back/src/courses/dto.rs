use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::courses::models::Course;

#[derive(Debug, Deserialize, Validate)]
pub struct CourseCreateDto {
    #[validate(length(
        min = 1,
        max = 150,
        message = "El nombre debe tener entre 1 y 150 caracteres"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 6,
        message = "El codigo debe tener entre 1 y 6 caracteres"
    ))]
    pub code: String,

    pub teacher_id: Uuid,

    #[validate(length(min = 1, message = "La contrasena es obligatoria"))]
    pub password: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "El estado debe tener entre 1 y 50 caracteres"
    ))]
    pub status: String,

    #[validate(range(min = 0, message = "La cantidad no puede ser negativa"))]
    pub quantity: i16,
}

#[derive(Debug, Serialize)]
pub struct CourseResponseDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub teacher_id: Uuid,
    pub password: String,
    pub status: String,
    pub quantity: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Course> for CourseResponseDto {
    fn from(value: Course) -> Self {
        Self {
            id: value.id,
            name: value.name,
            code: value.code,
            teacher_id: value.teacher_id,
            password: value.password,
            status: value.status,
            quantity: value.quantity,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
