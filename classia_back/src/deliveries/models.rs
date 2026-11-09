use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "delivery_status", rename_all = "lowercase")]
pub enum DeliveryStatus {
    Submitted,
    Replaced,
    Graded,
}

/// `file_url` is the server-generated `<uuid>.docx` name inside the deliveries
/// volume (DEC-03); it is never exposed through the API.
#[derive(Debug, sqlx::FromRow)]
pub struct Delivery {
    pub id: Uuid,
    pub activity_id: Uuid,
    pub student_id: Uuid,
    pub file_url: String,
    pub status: DeliveryStatus,
    pub version: i32,
    pub submitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `score` is stored as `NUMERIC(3, 1)`; queries must select it as
/// `score::float8` so it decodes without a decimal dependency.
#[derive(Debug, sqlx::FromRow)]
pub struct Evaluation {
    pub id: Uuid,
    pub delivery_id: Uuid,
    pub score: f64,
    pub feedback: Option<String>,
    pub teacher_id: Uuid,
    pub graded_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
