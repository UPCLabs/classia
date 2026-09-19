use uuid::Uuid;

use crate::users::models::User;
use crate::users::models::UserRole;
pub struct UserRepository {
    pool: sqlx::PgPool,
}

impl UserRepository {

    pub fn new(pool: sqlx::PgPool) -> Self {
        UserRepository { pool }
    }

    pub async fn get_user_by_id(&self, id_user: Uuid) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
              User,
              r#"
              SELECT
                  id_user,
                  name,
                  email,
                  password,
                  role AS "role: UserRole",
                  status,
                  token,
                  created_at,
                  updated_at
              FROM users
              WHERE id_user = $1
              "#,
              id_user,
          )
          .fetch_one(&self.pool)
          .await?;

        Ok(user)
    }
    
}

