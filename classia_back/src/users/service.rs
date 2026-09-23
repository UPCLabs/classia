use uuid::Uuid;
use validator::Validate;

use crate::{
    users::{
        dto::{ChangePasswordDto, UpdateUserDto, UserCreateDto},
        error::UserError,
        models::User,
        repository::UserRepository,
    },
    util::{password::hash_password, validation::validation_message},
};

pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn get_user_by_id(&self, id_user: Uuid) -> Result<User, UserError> {
        let consult = self
            .user_repository
            .get_user_by_id(id_user)
            .await
            .map_err(UserError::Database)?;

        match consult {
            Some(user) => Ok(user),
            None => Err(UserError::UserNotFound),
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, UserError> {
        let consult = self
            .user_repository
            .get_user_by_email(email)
            .await
            .map_err(UserError::Database)?;

        match consult {
            Some(user) => Ok(user),
            None => Err(UserError::UserNotFound),
        }
    }

    pub async fn create_user(&self, body: UserCreateDto) -> Result<User, UserError> {
        body.validate()
            .map_err(|errors| UserError::ValidationError(validation_message(errors)))?;

        let exists = self
            .user_repository
            .exists_by_email(&body.email)
            .await
            .map_err(UserError::Database)?;

        if exists {
            return Err(UserError::EmailAlreadyExists);
        }

        let password_hash = hash_password(&body.password)
            .map_err(|error| UserError::InternalError(error.to_string()))?;

        self.user_repository
            .insert_user(&body.name, &body.email, &password_hash, body.role)
            .await
            .map_err(UserError::Database)
    }

    pub async fn change_password(
        &self,
        id_user: Uuid,
        body: ChangePasswordDto,
    ) -> Result<(), UserError> {
        body.validate()
            .map_err(|errors| UserError::ValidationError(validation_message(errors)))?;

        let exists = self
            .user_repository
            .exists_by_id(id_user)
            .await
            .map_err(UserError::Database)?;

        if !exists {
            return Err(UserError::UserNotFound);
        }

        let new_password_hash = hash_password(&body.new_password)
            .map_err(|error| UserError::InternalError(error.to_string()))?;

        self.user_repository
            .update_password(id_user, &new_password_hash)
            .await
            .map_err(UserError::Database)
    }

    pub async fn update_user(&self, id_user: Uuid, body: UpdateUserDto) -> Result<User, UserError> {
        body.validate()
            .map_err(|errors| UserError::ValidationError(validation_message(errors)))?;

        if let Some(email) = &body.email {
            let existing = self
                .user_repository
                .get_user_by_email(email)
                .await
                .map_err(UserError::Database)?;

            if existing.is_some_and(|existing| existing.id != id_user) {
                return Err(UserError::EmailAlreadyExists);
            }
        }

        self.user_repository
            .update_user(id_user, body.name, body.email, body.role)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => UserError::UserNotFound,
                error => UserError::Database(error),
            })
    }

    pub async fn delete_user(&self, id_user: Uuid) -> Result<(), UserError> {
        self.user_repository
            .deactivate_user(id_user)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => UserError::UserNotFound,
                error => UserError::Database(error),
            })
    }

    pub async fn list_users(&self) -> Result<Vec<User>, UserError> {
        self.user_repository
            .list_users()
            .await
            .map_err(UserError::Database)
    }
}
