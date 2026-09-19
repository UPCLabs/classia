use uuid::Uuid;

use crate::users::{error::UserError, models::User, repository::UserRepository};

pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    // We need to change to our custom errors... API must not know about postgres
    // Here is an example, but we need to know, what way is better
    pub async fn get_user_by_id(&self, id_user: Uuid) -> Result<User, UserError> {
        self.user_repository
            .get_user_by_id(id_user)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => UserError::UserNotFound,
                error => UserError::Database(error),
            })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, UserError> {
        self.user_repository
            .get_user_by_email(email)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => UserError::UserNotFound,
                error => UserError::Database(error),
            })
    }
}
