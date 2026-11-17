use super::models::DeliveryRow;
use sqlx::{PgPool, query_as};
use uuid::Uuid;

const DELIVERY_DETAIL: &str = "
    SELECT d.id, d.activity_id, d.status, d.version, d.submitted_at, d.updated_at,
           u.id AS student_id, u.name AS student_name, u.email AS student_email,
           e.score::FLOAT8 AS score, e.feedback, e.graded_at,
           e.updated_at AS evaluation_updated_at
    FROM deliveries d
    JOIN users u ON u.id = d.student_id
    LEFT JOIN evaluations e ON e.delivery_id = d.id";

pub struct DeliveryRepository {
    pool: PgPool,
}
impl DeliveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_activity_and_student(
        &self,
        activity_id: Uuid,
        student_id: Uuid,
    ) -> Result<Option<DeliveryRow>, sqlx::Error> {
        query_as::<_, DeliveryRow>(&format!(
            "{DELIVERY_DETAIL} WHERE d.activity_id = $1 AND d.student_id = $2"
        ))
        .bind(activity_id)
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_by_activity(
        &self,
        activity_id: Uuid,
    ) -> Result<Vec<DeliveryRow>, sqlx::Error> {
        query_as::<_, DeliveryRow>(&format!(
            "{DELIVERY_DETAIL} WHERE d.activity_id = $1 ORDER BY d.updated_at ASC"
        ))
        .bind(activity_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_file_location(
        &self,
        delivery_id: Uuid,
    ) -> Result<Option<(Uuid, String)>, sqlx::Error> {
        sqlx::query_as::<_, (Uuid, String)>(
            "SELECT activity_id, file_url FROM deliveries WHERE id = $1",
        )
        .bind(delivery_id)
        .fetch_optional(&self.pool)
        .await
    }
}