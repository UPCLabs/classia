use super::models::{DeliveryRow, DeliveryStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct StudentDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct EvaluationDto {
    pub score: f64,
    pub feedback: Option<String>,
    pub graded_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DeliveryResponseDto {
    pub id: Uuid,
    pub activity_id: Uuid,
    pub student: StudentDto,
    pub status: DeliveryStatus,
    pub version: i32,
    pub submitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub evaluation: Option<EvaluationDto>,
}

impl From<DeliveryRow> for DeliveryResponseDto {
    fn from(value: DeliveryRow) -> Self {
        let evaluation = match (value.score, value.graded_at, value.evaluation_updated_at) {
            (Some(score), Some(graded_at), Some(updated_at)) => Some(EvaluationDto {
                score,
                feedback: value.feedback,
                graded_at,
                updated_at,
            }),
            _ => None,
        };

        Self {
            id: value.id,
            activity_id: value.activity_id,
            student: StudentDto {
                id: value.student_id,
                name: value.student_name,
                email: value.student_email,
            },
            status: value.status,
            version: value.version,
            submitted_at: value.submitted_at,
            updated_at: value.updated_at,
            evaluation,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EvaluationInputDto {
    pub score: f64,
    #[serde(default)]
    pub feedback: Option<String>,
    pub expected_version: i32,
}
