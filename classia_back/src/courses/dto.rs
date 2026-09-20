use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::courses::models::{Course};

#[derive(Debug, Serialize)]
pub struct CourseResponseDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub teacher_id: Uuid,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Course> for CourseResponseDto {
    fn from(value: Course) -> Self {
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

