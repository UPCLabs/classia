use super::models::{Activity, ActivityStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ActivityCreateDto {
    pub title: String,
    pub description: String,
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ActivityUpdateDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ActivityResponseDto {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    pub due_at: DateTime<Utc>,
    pub status: ActivityStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Activity> for ActivityResponseDto {
    fn from(value: Activity) -> Self {
        Self {
            id: value.id,
            course_id: value.course_id,
            title: value.title,
            description: value.description,
            due_at: value.due_at,
            status: value.status,
            published_at: value.published_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
