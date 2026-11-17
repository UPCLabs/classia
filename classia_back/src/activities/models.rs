use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "activity_status", rename_all = "lowercase")]
pub enum ActivityStatus {
    Draft,
    Published,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Activity {
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
