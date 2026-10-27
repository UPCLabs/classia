use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::activities::models::Activity;

pub struct ActivityRepository {
    pool: PgPool,
}

impl ActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, activity_id: Uuid) -> Result<Activity, sqlx::Error> {
        query_as::<_, Activity>("SELECT * FROM activities WHERE id = $1")
            .bind(activity_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn publish(&self, activity_id: Uuid) -> Result<Option<Activity>, sqlx::Error> {
        query_as::<_, Activity>(
            "UPDATE activities
             SET status = 'published',
                 published_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1 AND status = 'draft'
             RETURNING *",
        )
        .bind(activity_id)
        .fetch_optional(&self.pool)
        .await
    }
}
