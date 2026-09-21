use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Teacher,
    Student,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub status: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
