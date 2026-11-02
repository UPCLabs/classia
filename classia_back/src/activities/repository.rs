use super::{
    dto::{ActivityCreateDto, ActivityUpdateDto},
    models::Activity,
};
use sqlx::{PgPool, query_as};
use uuid::Uuid;

pub struct ActivityRepository {
    pool: PgPool,
}
impl ActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn create(
        &self,
        course_id: Uuid,
        body: ActivityCreateDto,
    ) -> Result<Activity, sqlx::Error> {
        let id = sqlx::query_scalar::<_, Uuid>("INSERT INTO activities (course_id, title, description, due_at) VALUES ($1, $2, $3, $4) RETURNING id")
            .bind(course_id).bind(body.title).bind(body.description).bind(body.due_at).fetch_one(&self.pool).await?;
        self.get(id).await
    }
    pub async fn get(&self, id: Uuid) -> Result<Activity, sqlx::Error> {
        query_as::<_, Activity>("SELECT id, course_id, title, description, due_at, status, published_at, created_at, updated_at FROM activities WHERE id = $1")
            .bind(id).fetch_one(&self.pool).await
    }
    pub async fn update(&self, id: Uuid, body: ActivityUpdateDto) -> Result<Activity, sqlx::Error> {
        sqlx::query("UPDATE activities SET title = COALESCE($1, title), description = COALESCE($2, description), due_at = COALESCE($3, due_at), updated_at = CURRENT_TIMESTAMP WHERE id = $4")
            .bind(body.title).bind(body.description).bind(body.due_at).bind(id).execute(&self.pool).await?;
        self.get(id).await
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

    pub async fn list_published_by_course(
        &self,
        course_id: Uuid,
    ) -> Result<Vec<Activity>, sqlx::Error> {
        query_as::<_, Activity>(
            "SELECT * FROM activities
             WHERE course_id = $1 AND status = 'published'
             ORDER BY due_at ASC",
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await
    }
}
