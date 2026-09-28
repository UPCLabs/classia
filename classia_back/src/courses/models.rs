use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::User;

#[derive(Debug, Deserialize, Serialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "course_status", rename_all = "lowercase")]
pub enum CourseStatus {
    Active,
    Inactive,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub teacher_id: Uuid,
    pub teacher_name: String,
    pub teacher_email: String,
    pub status: CourseStatus,
    pub capacity: i16,
    pub enrolled_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Enrollment {
    pub course_id: Uuid,
    pub student: User,
    pub enrolled_at: DateTime<Utc>,
}
