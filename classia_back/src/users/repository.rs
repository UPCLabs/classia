use sqlx::PgPool;
use sqlx::query_as;
use uuid::Uuid;

use crate::users::models::User;
use crate::users::models::UserRole;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn exists_by_email(&self, email: &str) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE email = $1
            )
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn exists_by_id(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE id = $1
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_user_by_id(&self, id_user: Uuid) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>(
            r#"
              SELECT
                  id,
                  name,
                  email,
                  password,
                  role,
                  status,
                  token,
                  created_at,
                  updated_at
              FROM users
              WHERE id = $1
              "#,
        )
        .bind(id_user)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_user_by_email(&self, user_email: &str) -> Result<Option<User>, sqlx::Error> {
        query_as::<_, User>(
            r#"
                SELECT
                    id,
                    name,
                    email,
                    password,
                    role,
                    status,
                    token,
                    created_at,
                    updated_at
                FROM users
                WHERE email = $1
            "#,
        )
        .bind(user_email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn insert_user(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Result<User, sqlx::Error> {
        let user = query_as::<_, User>(
            r#"
        INSERT INTO users (name, email, password, role, status, token)
        VALUES ( $1, $2, $3, $4, 'active', '')
        RETURNING
            id,
            name,
            email,
            password,
            role,
            status,
            token,
            created_at,
            updated_at
        "#,
        )
        .bind(name)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_password(&self, id_user: Uuid, new_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET password = $1, updated_at = now() WHERE id = $2")
            .bind(new_hash)
            .bind(id_user)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_user(
        &self,
        id_user: Uuid,
        name: Option<String>,
        email: Option<String>,
        role: Option<UserRole>,
    ) -> Result<User, sqlx::Error> {
        query_as::<_, User>(
            r#"
        UPDATE users
        SET name = COALESCE($1, name),
            email = COALESCE($2, email),
            role = COALESCE($3, role),
            updated_at = now()
        WHERE id = $4
        RETURNING id, name, email, password, role, status, token, created_at, updated_at
        "#,
        )
        .bind(name)
        .bind(email)
        .bind(role)
        .bind(id_user)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn deactivate_user(&self, id_user: Uuid) -> Result<(), sqlx::Error> {
        let result =
            sqlx::query("UPDATE users SET status = 'inactive', updated_at = now() WHERE id = $1")
                .bind(id_user)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }

    pub async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        query_as::<_, User>(
        "SELECT id, name, email, password, role, status, token, created_at, updated_at FROM users ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }
}
