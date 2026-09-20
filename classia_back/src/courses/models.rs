use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]

#[derive(Debug, sqlx::FromRow)]

pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub teacher_id: Uuid,
    pub password: String,    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
