use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]

pub struct Course {
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
