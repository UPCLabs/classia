use sqlx::PgPool;
use sqlx::query_as;
use uuid::Uuid;

use crate::users::models::User;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_user_by_id(&self, id_user: Uuid) -> Result<User, sqlx::Error> {
        let user = query_as::<_, User>(
            r#"
              SELECT
                  id,
                  name,
                  email,
                  password,
                  role AS "role: UserRole",
                  status,
                  token,
                  created_at,
                  updated_at
              FROM users
              WHERE id = $1
              "#,
        )
        .bind(id_user)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, user_email: &str) -> Result<User, sqlx::Error> {
        let user = query_as::<_, User>(
            r#"
                SELECT
                    id,
                    name,
                    email,
                    password,
                    role AS "role: UserRole",
                    status,
                    token,
                    created_at,
                    updated_at
                FROM users
                WHERE email = $1
            "#,
        )
        .bind(user_email)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
