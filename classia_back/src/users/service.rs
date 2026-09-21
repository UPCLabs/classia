use uuid::Uuid;
use validator::Validate;

use crate::{
    users::{dto::UserCreateDto, error::UserError, models::User, repository::UserRepository},
    util::password::hash_password,
};

pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

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

    pub async fn create_user(&self, body: UserCreateDto) -> Result<User, UserError> {
        let body = UserCreateDto {
            name: body.name.trim().to_string(),
            email: body.email.trim().to_string(),
            ..body
        };

        body.validate().map_err(|errors| {
            let mensaje = errors
                .field_errors()
                .values()
                .next() 
                .and_then(|field_errors| field_errors.first()) 
                .and_then(|error| error.message.clone()) 
                .unwrap_or_else(|| "Datos inválidos".into());
            UserError::ValidationError(mensaje.to_string())
        })?;

        if self
            .user_repository
            .get_user_by_email(&body.email)
            .await
            .is_ok()
        {
            return Err(UserError::EmailAlreadyExists);
        }

        let password_hash = hash_password(&body.password).map_err(|_| UserError::HashingError)?;

        self.user_repository
            .insert_user(&body.name, &body.email, &password_hash, body.role)
            .await
            .map_err(UserError::Database)
    }
}
