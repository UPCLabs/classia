use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::courses::models::Course;

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
