use super::models::DeliveryRow;
use sqlx::{PgConnection, PgPool, query_as};
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
/// Row locked before replacing a delivery (DEC-04). `open` comes from the
/// database clock, as the contract requires.
#[derive(Debug, sqlx::FromRow)]
pub struct ReplaceTarget {
    pub student_id: Uuid,
    pub version: i32,
    pub file_url: String,
    pub open: bool,
    pub score: Option<f64>,
    pub feedback: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PreviousEvaluation {
    pub score: f64,
    pub feedback: Option<String>,
}

impl DeliveryRepository {
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn find_detail(&self, delivery_id: Uuid) -> Result<Option<DeliveryRow>, sqlx::Error> {
        query_as::<_, DeliveryRow>(&format!("{DELIVERY_DETAIL} WHERE d.id = $1"))
            .bind(delivery_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_owner(&self, delivery_id: Uuid) -> Result<Option<(Uuid, Uuid)>, sqlx::Error> {
        sqlx::query_as::<_, (Uuid, Uuid)>(
            "SELECT activity_id, student_id FROM deliveries WHERE id = $1",
        )
        .bind(delivery_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn activity_is_open(
        connection: &mut PgConnection,
        activity_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT due_at > now() FROM activities WHERE id = $1")
            .bind(activity_id)
            .fetch_one(connection)
            .await
    }

    pub async fn insert(
        connection: &mut PgConnection,
        activity_id: Uuid,
        student_id: Uuid,
        file_url: &str,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO deliveries (activity_id, student_id, file_url)
             VALUES ($1, $2, $3)
             RETURNING id",
        )
        .bind(activity_id)
        .bind(student_id)
        .bind(file_url)
        .fetch_one(connection)
        .await
    }

    pub async fn lock_for_replace(
        connection: &mut PgConnection,
        delivery_id: Uuid,
    ) -> Result<Option<ReplaceTarget>, sqlx::Error> {
        query_as::<_, ReplaceTarget>(
            "SELECT d.student_id, d.version, d.file_url, a.due_at > now() AS open,
                    e.score::FLOAT8 AS score, e.feedback
             FROM deliveries d
             JOIN activities a ON a.id = d.activity_id
             LEFT JOIN evaluations e ON e.delivery_id = d.id
             WHERE d.id = $1
             FOR UPDATE OF d",
        )
        .bind(delivery_id)
        .fetch_optional(connection)
        .await
    }

    pub async fn replace_file(
        connection: &mut PgConnection,
        delivery_id: Uuid,
        file_url: &str,
    ) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar(
            "UPDATE deliveries
             SET file_url = $2, status = 'replaced', version = version + 1,
                 updated_at = now()
             WHERE id = $1
             RETURNING version",
        )
        .bind(delivery_id)
        .bind(file_url)
        .fetch_one(connection)
        .await
    }

    pub async fn delete_evaluation(
        connection: &mut PgConnection,
        delivery_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM evaluations WHERE delivery_id = $1")
            .bind(delivery_id)
            .execute(connection)
            .await?;
        Ok(())
    }

    /// Optimistic lock of DEC-04: no row means another change won.
    pub async fn mark_graded(
        connection: &mut PgConnection,
        delivery_id: Uuid,
        expected_version: i32,
    ) -> Result<Option<i32>, sqlx::Error> {
        sqlx::query_scalar(
            "UPDATE deliveries
             SET version = version + 1, status = 'graded', updated_at = now()
             WHERE id = $1 AND version = $2
             RETURNING version",
        )
        .bind(delivery_id)
        .bind(expected_version)
        .fetch_optional(connection)
        .await
    }

    pub async fn lock_evaluation(
        connection: &mut PgConnection,
        delivery_id: Uuid,
    ) -> Result<Option<PreviousEvaluation>, sqlx::Error> {
        query_as::<_, PreviousEvaluation>(
            "SELECT score::FLOAT8 AS score, feedback
             FROM evaluations WHERE delivery_id = $1
             FOR UPDATE",
        )
        .bind(delivery_id)
        .fetch_optional(connection)
        .await
    }

    pub async fn upsert_evaluation(
        connection: &mut PgConnection,
        delivery_id: Uuid,
        teacher_id: Uuid,
        score: f64,
        feedback: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO evaluations (delivery_id, score, feedback, teacher_id)
             VALUES ($1, $2::FLOAT8::NUMERIC(3, 1), $3, $4)
             ON CONFLICT (delivery_id) DO UPDATE
             SET score = EXCLUDED.score, feedback = EXCLUDED.feedback,
                 teacher_id = EXCLUDED.teacher_id, updated_at = now()",
        )
        .bind(delivery_id)
        .bind(score)
        .bind(feedback)
        .bind(teacher_id)
        .execute(connection)
        .await?;
        Ok(())
    }
}
